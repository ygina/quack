#include <cassert>       // for assert
#include <chrono>        // for std::chrono::high_resolution_clock
#include <cstddef>       // for std::size_t
#include <cstdint>       // for std::uint16_t, std::uint32_t, std::uint64_t
#include <iostream>      // for std::cout
#include <limits>        // for std::numeric_limits
#include <random>        // for std::random_device, std::mt19937_64
#include <string>        // for std::string
#include <unordered_map> // for std::unordered_map
#include <vector>        // for std::vector

#include "PowerSumAccumulator.hpp"
#include "MonicPolynomialEvaluator.hpp"


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


static const auto power_tables = power_tables_16<UINT16_C(65'521)>(50);


// How long does it take to insert a number into a PowerSumAccumulator?
template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
void benchmark_insertion(
    std::size_t size,
    std::size_t num_packets,
    std::size_t num_drop,
    std::size_t num_trials
) {

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<T_NARROW> dist(
        std::numeric_limits<T_NARROW>::min(),
        std::numeric_limits<T_NARROW>::max()
    );

    // Allocate buffer for benchmark durations.
    std::vector<uint32_t> durations;

    for (std::size_t i = 0; i < num_trials + 1; ++i) {

        // Generate <num_packets> + 10 random numbers.
        std::vector<T_NARROW> numbers;
        // for (std::size_t j = 0; j < num_packets + 10; ++j)
        for (std::size_t j = 0; j < num_packets; ++j)
            numbers.push_back(dist(gen));

        // Construct two empty PowerSumAccumulators.
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc1(size);
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc2(size);

        // // Warm up the instruction cache by inserting a few numbers.
        // for (std::size_t i = num_packets; i < num_packets + 10; ++i) {
        //     acc1.insert(numbers[i]);
        // }

        // Insert a bunch of random numbers into the accumulator.
        begin_timer();
        for (std::size_t j = 0; j < num_packets; ++j)
            acc1.insert(numbers[j]);
        for (std::size_t j = 0; j < num_packets - num_drop; ++j)
            acc2.insert(numbers[j]);
        do_not_discard(acc1);
        do_not_discard(acc2);
        end_timer();

        if (i > 0) {
            auto duration = print_timer("Insert " + std::to_string(num_packets)
                        + " numbers into 2 PowerSumAccumulators (" +
                        std::string(TYPE_NAME<T_NARROW>) + ", threshold = " +
                        std::to_string(size) + ")");
            durations.push_back(duration);
        }
    }
    print_summary(durations);
}


void run_insertion_benchmark(
    std::size_t threshold,
    std::size_t num_packets,
    std::size_t num_bits_id,
    std::size_t num_drop,
    std::size_t num_trials
) {
    if (num_bits_id == 16) {
        benchmark_insertion<std::uint16_t, std::uint32_t,
            UINT16_C(65'521)>(threshold, num_packets, num_drop, num_trials);
    } else if (num_bits_id == 32) {
        benchmark_insertion<std::uint32_t, std::uint64_t,
            UINT32_C(4'294'967'291)>(threshold, num_packets, num_drop, num_trials);
    } else if (num_bits_id == 64) {
        benchmark_insertion<std::uint64_t, __uint128_t,
            UINT64_C(18'446'744'073'709'551'557)>(threshold, num_packets, num_drop, num_trials);
    } else {
        std::cout << "ERROR: <num_bits_id> must be 16, 32, or 64" << std::endl;
    }
}


////////////////////////////////////////////////////////////////////////////////


// How long does it take to compute the set-theoretic difference between two
// PowerSumAccumulators, assuming one is a subset of the other?
template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
void benchmark_decode(
    std::size_t size,
    std::size_t num_packets,
    std::size_t num_drop,
    std::size_t num_trials
) {

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<T_NARROW> dist(
        std::numeric_limits<T_NARROW>::min(),
        std::numeric_limits<T_NARROW>::max()
    );

    // Allocate buffer for benchmark durations.
    std::vector<uint32_t> durations;

    using evaluator = MonicPolynomialEvaluator<T_NARROW, T_WIDE, MODULUS>;

    for (std::size_t i = 0; i < num_trials + 1; ++i) {
        // Allocate variable for counting false positives.
        std::size_t fp = 0;

        // Generate 1000 random numbers.
        std::vector<T_NARROW> numbers;
        for (std::size_t j = 0; j < num_packets; ++j) {
            numbers.push_back(dist(gen));
        }

        // Construct two empty PowerSumAccumulators.
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc_1(size);
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc_2(size);

        // Insert all random numbers into the first accumulator.
        for (std::size_t j = 0; j < num_packets; ++j) {
            acc_1.insert(numbers[j]);
        }

        // Insert all but num_drop random numbers into the second accumulator.
        for (std::size_t j = 0; j < num_packets - num_drop; ++j) {
            acc_2.insert(numbers[j]);
        }

        // Pre-allocate buffer for polynomial coefficients.
        std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>> coeffs(num_drop);

        // Allocate buffer for missing packets.
        std::vector<T_NARROW> dropped;

        begin_timer();
        if (num_drop > 0) {
            acc_1 -= acc_2;
            acc_1.to_polynomial_coefficients(coeffs);
            for (std::size_t j = 0; j < num_packets - num_drop; ++j) {
                const auto value = evaluator::eval(coeffs, numbers[j]);
                if (!value) fp++;
                do_not_discard(value);
            }
            for (std::size_t j = num_packets - num_drop; j < num_packets; ++j) {
                const auto value = evaluator::eval(coeffs, numbers[j]);
                assert(!value);
                do_not_discard(value);
                dropped.push_back(numbers[j]);
            }
        }
        do_not_discard(dropped);
        end_timer();

        if (i > 0) {
            auto duration =
                print_timer("Decode time (" + std::string(TYPE_NAME<T_NARROW>) +
                        ", threshold = " + std::to_string(size) +
                        ", num_packets = " + std::to_string(num_packets) +
                        ", false_positives = " + std::to_string(fp) +
                        ", dropped = " + std::to_string(num_drop) + ")");
            durations.push_back(duration);
        }
    }

    print_summary(durations);
}


void run_decode_benchmark(
    std::size_t threshold,
    std::size_t num_packets,
    std::size_t num_bits_id,
    std::size_t num_drop,
    std::size_t num_trials
) {
    if (num_bits_id == 16) {
        benchmark_decode<std::uint16_t, std::uint32_t,
            UINT16_C(65'521)>(threshold, num_packets, num_drop, num_trials);
    } else if (num_bits_id == 32) {
        benchmark_decode<std::uint32_t, std::uint64_t,
            UINT32_C(4'294'967'291)>(threshold, num_packets, num_drop, num_trials);
    } else if (num_bits_id == 64) {
        benchmark_decode<std::uint64_t, __uint128_t,
            UINT64_C(18'446'744'073'709'551'557)>(threshold, num_packets, num_drop, num_trials);
    } else {
        std::cout << "ERROR: <num_bits_id> must be 16, 32, or 64" << std::endl;
        return;
    }
}


////////////////////////////////////////////////////////////////////////////////


int main(int argc, char **argv) {

    std::size_t threshold = 20;
    std::size_t num_packets = 1000;
    std::size_t num_bits_id = 16;
    std::size_t num_drop = threshold;
    std::size_t num_trials = 10;
    bool benchmark_insertion = false;
    bool benchmark_decode = false;

    for (int i = 0; i < argc; ++i) {
        if (std::string(argv[i]) == "-t") {
            if (i + 1 < argc) {
                threshold = std::stoull(argv[i + 1]);
                num_drop = threshold;
                ++i;
            }
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
        } else if (std::string(argv[i]) == "--insertion") {
            benchmark_insertion = true;
        } else if (std::string(argv[i]) == "--decode") {
            benchmark_decode = true;
        }
    }

    if (benchmark_insertion ^ benchmark_decode) {
        if (benchmark_insertion) {
            run_insertion_benchmark(
                threshold,
                num_packets,
                num_bits_id,
                num_drop,
                num_trials
            );
        } else if (benchmark_decode) {
            run_decode_benchmark(
                threshold,
                num_packets,
                num_bits_id,
                num_drop,
                num_trials
            );
        }
    } else {
        std::cout << "Usage: " << argv[0] << " [-t <threshold>] "
                  << "[-n <num_packets>] " << "[-b <num_bits_id>] "
                  << "[--dropped <num_drop>] "
                  << "[--trials <num_trials>] [--insertion] [--decode]"
                  << std::endl;
    }

    return 0;
}
