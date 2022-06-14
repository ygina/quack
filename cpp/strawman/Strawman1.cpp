#include <cassert>       // for assert
#include <chrono>        // for std::chrono::high_resolution_clock
#include <cstddef>       // for std::size_t
#include <cstdint>       // for std::uint16_t, std::uint32_t, std::uint64_t
#include <iostream>      // for std::cout
#include <limits>        // for std::numeric_limits
#include <random>        // for std::random_device, std::mt19937_64
#include <string>        // for std::string
#include <vector>        // for std::vector
#include <set>           // for std::multiset


///////////////////////////////////////////////////////// BENCHMARKING UTILITIES


static std::chrono::high_resolution_clock::time_point begin_time;
static std::chrono::high_resolution_clock::time_point end_time;


static void begin_timer() {
    begin_time = std::chrono::high_resolution_clock::now();
}


static void end_timer() {
    end_time = std::chrono::high_resolution_clock::now();
}


static auto print_timer(const std::string &message) {
    using std::chrono::nanoseconds;
    const auto duration = std::chrono::duration_cast<nanoseconds>(
        end_time - begin_time
    ).count();
    std::cout << message << ": " << duration << " ns" << std::endl;
    return duration;
}

static void print_summary(std::vector<uint32_t> d) {
    uint32_t avg;
    if (d.empty()) {
        avg = 0;
    } else {
        avg = std::reduce(d.begin(), d.end(), 0.0) / d.size();
    }
    std::cout << "SUMMARY: "
              << "num_trials = " << std::to_string(d.size())
              << ", avg = " << std::to_string(avg) << " ns"
              << std::endl;
}


// A trick from the Google Benchmark library: this prevents dead code
// elimination on a variable that is calculated but never used.
template <typename T>
void do_not_discard(const T &value) {
    asm volatile("" : : "r,m"(value) : "memory");
}


template <typename T>
static constexpr const char *TYPE_NAME = "UNKNOWN";
template <> constexpr const char *TYPE_NAME<std::uint16_t> = "16-bit integers";
template <> constexpr const char *TYPE_NAME<std::uint32_t> = "32-bit integers";
template <> constexpr const char *TYPE_NAME<std::uint64_t> = "64-bit integers";


////////////////////////////////////////////////////////////////////////////////


// How long does it take to compute the set-theoretic difference between two
// PowerSumAccumulators, assuming one is a subset of the other?
template <typename T_BITS>
void benchmark_decode(
    std::size_t num_packets,
    std::size_t num_drop,
    std::size_t num_trials
) {

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<T_BITS> dist(
        std::numeric_limits<T_BITS>::min(),
        std::numeric_limits<T_BITS>::max()
    );

    // Allocate buffer for benchmark durations.
    std::vector<uint32_t> durations;

    for (std::size_t i = 0; i < num_trials + 1; ++i) {
        // Generate <num_packets> random numbers.
        std::vector<T_BITS> sender;
        std::vector<T_BITS> receiver;
        for (std::size_t j = 0; j < num_packets - num_drop; ++j) {
            sender.push_back(dist(gen));
            receiver.push_back(dist(gen));
        }
        for (std::size_t j = num_packets - num_drop; j < num_packets; ++j) {
            sender.push_back(dist(gen));
        }

        // Initialize buffers.
        std::multiset<T_BITS> sender_mset;
        std::multiset<T_BITS> receiver_mset;
        std::multiset<T_BITS> difference;

        begin_timer();
        if (num_drop > 0) {
            for (std::size_t j = 0; j < sender.size(); j++)
                sender_mset.insert(sender[j]);
            for (std::size_t j = 0; j < receiver.size(); j++)
                receiver_mset.insert(receiver[j]);
            std::set_difference(std::make_move_iterator(sender.begin()),
                                std::make_move_iterator(sender.end()),
                                receiver.begin(), receiver.end(),
                                std::inserter(difference, difference.begin()));
            do_not_discard(difference);
        }
        end_timer();

        if (i > 0) {
            auto duration =
                print_timer("Decode time (" + std::string(TYPE_NAME<T_BITS>) +
                        ", num_packets = " + std::to_string(num_packets) +
                        ", dropped = " + std::to_string(num_drop) + ")");
            durations.push_back(duration);
        }
    }

    print_summary(durations);
}


void run_decode_benchmark(
    std::size_t num_packets,
    std::size_t num_bits_id,
    std::size_t num_drop,
    std::size_t num_trials
) {
    if (num_bits_id == 16) {
        benchmark_decode<std::uint16_t>(num_packets, num_drop, num_trials);
    } else if (num_bits_id == 32) {
        benchmark_decode<std::uint32_t>(num_packets, num_drop, num_trials);
    } else if (num_bits_id == 64) {
        benchmark_decode<std::uint64_t>(num_packets, num_drop, num_trials);
    } else {
        std::cout << "ERROR: <num_bits_id> must be 16, 32, or 64" << std::endl;
        return;
    }
}


////////////////////////////////////////////////////////////////////////////////


int main(int argc, char **argv) {

    std::size_t num_packets = 1000;
    std::size_t num_bits_id = 16;
    std::size_t num_drop = 20;
    std::size_t num_trials = 10;
    bool help = false;

    for (int i = 0; i < argc; ++i) {
        if (std::string(argv[i]) == "-h" || std::string(argv[i]) == "help") {
            help = true;
            break;
        } else if (std::string(argv[i]) == "-n") {
            if (i + 1 < argc) {
                num_packets = std::stoull(argv[i + 1]);
                ++i;
            }
        } else if (std::string(argv[i]) == "-b") {
            if (i + 1 < argc) {
                num_bits_id = std::stoull(argv[i + 1]);
                ++i;
            }
        } else if (std::string(argv[i]) == "--trials") {
            if (i + 1 < argc) {
                num_trials = std::stoull(argv[i + 1]);
                ++i;
            }
        } else if (std::string(argv[i]) == "--dropped") {
            if (i + 1 < argc) {
                num_drop = std::stoull(argv[i + 1]);
                ++i;
            }
        }
    }

    if (!help) {
        run_decode_benchmark(
            num_packets,
            num_bits_id,
            num_drop,
            num_trials
        );
    } else {
        std::cout << "Usage: " << argv[0] << "[-n <num_packets>] "
                  << "[-b <num_bits_id>] " << "[--dropped <num_drop>] "
                  << "[--trials <num_trials>]"
                  << std::endl;
    }

    return 0;
}
