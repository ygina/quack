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


static void print_timer(const std::string &message) {
    using std::chrono::nanoseconds;
    const auto duration = std::chrono::duration_cast<nanoseconds>(
        end_time - begin_time
    ).count();
    std::cout << message << ": " << duration << " ns" << std::endl;
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
void benchmark_insertion(std::size_t size, std::size_t num_trials) {

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<T_NARROW> dist(
        std::numeric_limits<T_NARROW>::min(),
        std::numeric_limits<T_NARROW>::max()
    );

    for (std::size_t i = 0; i < num_trials + 1; ++i) {

        // Generate 1010 random numbers.
        std::vector<T_NARROW> numbers;
        for (std::size_t j = 0; j < 1010; ++j) {
            numbers.push_back(dist(gen));
        }

        // Construct an empty PowerSumAccumulator.
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc(size);

        // Warm up the instruction cache by inserting a few numbers.
        for (std::size_t i = 1000; i < 1010; ++i) {
            acc.insert(numbers[i]);
        }

        // Insert a bunch of random numbers into the accumulator.
        begin_timer();
        for (std::size_t j = 0; j < 1000; ++j) {
            acc.insert(numbers[i]);
            do_not_discard(acc);
        }
        end_timer();

        if (i > 0) {
            print_timer("Insert 1000 numbers into PowerSumAccumulator (" +
                        std::string(TYPE_NAME<T_NARROW>) + ", threshold = " +
                        std::to_string(size) + ")");
        }
    }
}


void run_insertion_benchmark(std::size_t max_size, std::size_t num_trials) {
    for (std::size_t i = 1; i <= max_size; ++i) {
        benchmark_insertion<std::uint16_t, std::uint32_t,
                            UINT16_C(65'521)>(i, num_trials);
    }
    for (std::size_t i = 1; i <= max_size; ++i) {
        benchmark_insertion<std::uint32_t, std::uint64_t,
                            UINT32_C(4'294'967'291)>(i, num_trials);
    }
    for (std::size_t i = 1; i <= max_size; ++i) {
        benchmark_insertion<std::uint64_t, __uint128_t,
                            UINT64_C(18'446'744'073'709'551'557)>(i, num_trials);
    }
}


int main() {
    run_insertion_benchmark(50, 100);
}


////////////////////////////////////////////////////////////////////////////////


// How long does it take to compute the set-theoretic difference between two
// PowerSumAccumulators, assuming one is a subset of the other?
template <typename T_NARROW, typename T_WIDE, T_NARROW MODULUS>
void benchmark_decode(
    std::size_t size,
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

    using evaluator = MonicPolynomialEvaluator<T_NARROW, T_WIDE, MODULUS>;

    for (std::size_t i = 0; i < num_trials + 1; ++i) {

        // Generate 1000 random numbers.
        std::vector<T_NARROW> numbers;
        for (std::size_t j = 0; j < 1000; ++j) {
            numbers.push_back(dist(gen));
        }

        // Construct two empty PowerSumAccumulators.
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc_1(size);
        PowerSumAccumulator<T_NARROW, T_WIDE, MODULUS> acc_2(size);

        // Insert all random numbers into the first accumulator.
        for (std::size_t j = 0; j < 1000; ++j) { acc_1.insert(numbers[j]); }

        // Insert all but num_drop random numbers into the second accumulator.
        for (std::size_t j = 0; j < 1000 - num_drop; ++j) {
            acc_2.insert(numbers[j]);
        }

        // Pre-allocate buffer for polynomial coefficients.
        std::vector<ModularInteger<T_NARROW, T_WIDE, MODULUS>> coeffs(num_drop);

        begin_timer();
        if (num_drop > 0) {
            acc_1 -= acc_2;
            acc_1.to_polynomial_coefficients(coeffs);
            for (std::size_t j = 0; j < 1000 - num_drop; ++j) {
                const auto value = evaluator::eval(coeffs, numbers[j]);
                do_not_discard(value);
            }
            for (std::size_t j = 1000 - num_drop; j < 1000; ++j) {
                const auto value = evaluator::eval(coeffs, numbers[j]);
                assert(!value);
                do_not_discard(value);
            }
        }
        end_timer();

        if (i > 0) {
            print_timer("Decode time (" + std::string(TYPE_NAME<T_NARROW>) +
                        ", threshold = " + std::to_string(size) +
                        ", dropped = " + std::to_string(num_drop) + ")");
        }
    }
}


void run_decode_benchmark(std::size_t size, std::size_t num_trials) {
    for (std::size_t i = 0; i <= size; ++i) {
        benchmark_decode<std::uint16_t, std::uint32_t,
                         UINT16_C(65'521)>(size, i, num_trials);
    }
    for (std::size_t i = 0; i <= size; ++i) {
        benchmark_decode<std::uint32_t, std::uint64_t,
                         UINT32_C(4'294'967'291)>(size, i, num_trials);
    }
    for (std::size_t i = 0; i <= size; ++i) {
        benchmark_decode<std::uint64_t, __uint128_t,
                         UINT64_C(18'446'744'073'709'551'557)>(size, i, num_trials);
    }
}


////////////////////////////////////////////////////////////////////////////////


// int main(int argc, char **argv) {

//     std::size_t threshold = 32;
//     std::size_t num_trials = 10;

//     for (int i = 0; i < argc; ++i) {
//         if (std::string(argv[i]) == "--threshold") {
//             if (i + 1 < argc) {
//                 threshold = std::stoull(argv[i + 1]);
//                 ++i;
//             }
//         } else if (std::string(argv[i]) == "--trials") {
//             if (i + 1 < argc) {
//                 num_trials = std::stoull(argv[i + 1]);
//                 ++i;
//             }
//         }
//     }

//     run_decode_benchmark(threshold, num_trials);
// }
