#include <algorithm> // for std::count
#include <cassert>   // for assert
#include <chrono>    // for std::chrono::high_resolution_clock
#include <cstddef>   // std::size_t
#include <cstdint>   // for std::uint16_t, std::uint32_t, std::uint64_t
#include <iostream>  // for std::cout
#include <limits>    // for std::numeric_limits
#include <random>    // for std::random_device, std::mt19937_64
#include <string>    // for std::string
#include <unordered_map> // for std::unordered_map
#include <vector>    // for std::vector

#include "PowerSumAccumulator.hpp"
#include "MonicPolynomialEvaluator.hpp"


// static std::chrono::high_resolution_clock::time_point begin_time;


// static void begin_timer() {
//     begin_time = std::chrono::high_resolution_clock::now();
// }


// static void end_timer(const std::string &message) {
//     using std::chrono::duration_cast;
//     using std::chrono::nanoseconds;
//     const auto end_time = std::chrono::high_resolution_clock::now();
//     const auto duration = duration_cast<nanoseconds>(end_time - begin_time).count();
//     std::cout << message << ": " << duration << " ns" << std::endl;
// }


template <typename T>
bool is_subset(const std::vector<T> &a, const std::vector<T> &b) {
    std::unordered_map<T, std::size_t> a_counts, b_counts;
    for (const auto &x : a) { ++a_counts[x]; }
    for (const auto &x : b) { ++b_counts[x]; }
    for (const auto &x : a) {
        if (a_counts[x] > b_counts[x]) { return false; }
    }
    return true;
}


int main() {

    constexpr std::size_t NUM_PACKETS_SENT = 1000;
    constexpr double DROP_PROBABILITY = 0.02;
    constexpr std::size_t DIGEST_SIZE = 32;

    // using packet_t = std::uint32_t;
    // using wide_t = std::uint64_t;
    // // This is the largest prime number that fits in a 32-bit unsigned integer.
    // constexpr packet_t MODULUS = 4'294'967'291; // 2^32 - 5

    using packet_t = std::uint16_t;
    using wide_t = std::uint32_t;
    // This is the largest prime number that fits in a 16-bit unsigned integer.
    constexpr packet_t MODULUS = 65'521; // 2^16 - 5

    using packet_limits = std::numeric_limits<packet_t>;
    using accumulator_t = PowerSumAccumulator<packet_t, wide_t,
                                              MODULUS, DIGEST_SIZE>;
    using evaluator_t = MonicPolynomialEvaluator<packet_t, wide_t,
                                                 MODULUS, DIGEST_SIZE>;

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<packet_t> packet_dist(packet_limits::min(),
                                                        packet_limits::max());
    std::uniform_real_distribution<double> drop_dist(0.0, 1.0);

    // Initialize packet data structures.
    std::vector<packet_t> sent_packets(NUM_PACKETS_SENT);
    std::vector<packet_t> received_packets;
    std::vector<packet_t> resent_packets;
    received_packets.reserve(NUM_PACKETS_SENT);
    resent_packets.reserve(NUM_PACKETS_SENT);
    accumulator_t server_acc;
    accumulator_t middlebox_acc;

    std::uint64_t num_successful_recoveries = 0;
    std::uint64_t num_erroneous_recoveries = 0;
    std::uint64_t num_correct_failures = 0;

    while (true) {

        // Generate uniformly random packets for the server to send.
        for (std::size_t i = 0; i < NUM_PACKETS_SENT; ++i) {
            const packet_t packet = packet_dist(gen);
            sent_packets[i] = packet;
            server_acc.insert(packet);
        }

        // Generate list of packets that the middlebox receives, with
        // simulated packet loss between the server and the middlebox.
        for (const auto &packet : sent_packets) {
            if (drop_dist(gen) >= DROP_PROBABILITY) {
                received_packets.push_back(packet);
                middlebox_acc.insert(packet);
            }
        }
        // const std::size_t num_dropped_packets =
        //     sent_packets.size() - received_packets.size();

        // At this point, the middlebox sends its subset digest back to the
        // server. The server takes the difference of its own digest and the
        // middlebox's digest. It then uses that difference to construct the
        // coefficients of a polynomial p, whose roots are the packets that
        // were dropped.
        server_acc -= middlebox_acc;
        const auto coeffs = server_acc.to_polynomial_coefficients();

        // The server checks whether each of the packets it sent is a root of
        // this polynomial p. If so, the server adds that packet to a queue of
        // packets to be re-sent to the middlebox.
        for (const auto &packet : sent_packets) {
            if (!evaluator_t::eval(coeffs, packet)) {
                resent_packets.push_back(packet);
            }
        }

        // By looking at the number of trailing zeroes in the list of
        // coefficients of p, the server can determine a lower bound on the
        // number of packets that were dropped on the way to the middlebox.
        const std::size_t num_expected = DIGEST_SIZE - count_trailing_zeros(coeffs);

        // We check whether this lower bound is consistent with the number of
        // packets placed in the re-send queue.
        if (resent_packets.size() < num_expected) {

            // If the number of packets in the re-send queue is smaller than
            // our computed lower bound, then we conclude that the digests are
            // not telling the whole story, i.e., more than `DIGEST_SIZE`
            // packets were dropped between the server and middlebox.

            // This is an unrecoverable error state. In this situation, the
            // server re-sends everything to the middlebox.
            ++num_correct_failures;
            // std::cout << "All is lost; re-send everything!" << std::endl;

            // This error state should only occur when the number of packets
            // dropped between the server and the middlebox exceeds the digest
            // size.
            assert(received_packets.size() + DIGEST_SIZE < NUM_PACKETS_SENT);

        } else {

            // If the number of packets in the re-send queue is consistent with
            // our computed lower bound, then we go ahead and re-send those
            // packets. (For simplicity, we assume that none of the re-sent
            // packets are dropped.)
            received_packets.insert(received_packets.end(),
                                    resent_packets.begin(), resent_packets.end());

            // At this point, the (multi)set of received packets should be a
            // superset of the (multi)set of sent packets. (The sets may not be
            // strictly equal, because some packets may have been unnecessarily
            // re-sent due to hash collisions.)
            if (is_subset(sent_packets, received_packets)) {
                ++num_successful_recoveries;
            } else {
                ++num_erroneous_recoveries;
            }

            // std::cout << "Successful recovery from "
            //           << num_dropped_packets << " dropped packets (sent "
            //           << received_packets.size() - sent_packets.size()
            //           << " extra packets)." << std::endl;

        }

        received_packets.clear();
        received_packets.reserve(NUM_PACKETS_SENT);
        resent_packets.clear();
        resent_packets.reserve(NUM_PACKETS_SENT);
        server_acc.clear();
        middlebox_acc.clear();

        const std::uint64_t num_trials = num_successful_recoveries +
                                         num_erroneous_recoveries +
                                         num_correct_failures;
        if (num_trials % 5000 == 0) {
            std::cout << "Completed " << num_trials << " trials ["
                      << num_successful_recoveries << " successful recoveries, "
                      << num_erroneous_recoveries << " erroneous recoveries, "
                      << num_correct_failures << " correct failures]."
                      << std::endl;
        }

    }

}
