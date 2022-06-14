#include <cassert>       // for assert
#include <chrono>        // for std::chrono::high_resolution_clock
#include <cstddef>       // for std::size_t
#include <cstdint>       // for std::uint16_t, std::uint32_t, std::uint64_t
#include <iostream>      // for std::cout
#include <limits>        // for std::numeric_limits
#include <random>        // for std::random_device, std::mt19937_64
#include <string>        // for std::string
#include <vector>        // for std::vector
#include <algorithm>     // for std::min

/* Include the GCC super header */
#if defined(__GNUC__)
# include <stdint.h>
# include <x86intrin.h>
#endif

/* Microsoft supports Intel SHA ACLE extensions as of Visual Studio 2015 */
#if defined(_MSC_VER)
# include <immintrin.h>
# define WIN32_LEAN_AND_MEAN
# include <Windows.h>
typedef UINT32 uint32_t;
typedef UINT8 uint8_t;
#endif

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

static uint32_t print_summary(std::vector<uint32_t> d) {
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
    return avg;
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

/* sha256-x86.c - Intel SHA extensions using C intrinsics  */
/*   Written and place in public domain by Jeffrey Walton  */
/*   Based on code from Intel, and by Sean Gulley for      */
/*   the miTLS project.                                    */
///////////////////////////////////////////////////////////////////////// SHA256

/* Process multiple blocks. The caller is responsible for setting the initial */
/*  state, and the caller is responsible for padding the final block.        */
void sha256_process_x86(uint32_t state[8], const uint8_t data[], uint32_t length)
{
    __m128i STATE0, STATE1;
    __m128i MSG, TMP;
    __m128i MSG0, MSG1, MSG2, MSG3;
    __m128i ABEF_SAVE, CDGH_SAVE;
    const __m128i MASK = _mm_set_epi64x(0x0c0d0e0f08090a0bULL, 0x0405060700010203ULL);

    /* Load initial values */
    TMP = _mm_loadu_si128((const __m128i*) &state[0]);
    STATE1 = _mm_loadu_si128((const __m128i*) &state[4]);


    TMP = _mm_shuffle_epi32(TMP, 0xB1);          /* CDAB */
    STATE1 = _mm_shuffle_epi32(STATE1, 0x1B);    /* EFGH */
    STATE0 = _mm_alignr_epi8(TMP, STATE1, 8);    /* ABEF */
    STATE1 = _mm_blend_epi16(STATE1, TMP, 0xF0); /* CDGH */

    while (length >= 64)
    {
        /* Save current state */
        ABEF_SAVE = STATE0;
        CDGH_SAVE = STATE1;

        /* Rounds 0-3 */
        MSG = _mm_loadu_si128((const __m128i*) (data+0));
        MSG0 = _mm_shuffle_epi8(MSG, MASK);
        MSG = _mm_add_epi32(MSG0, _mm_set_epi64x(0xE9B5DBA5B5C0FBCFULL, 0x71374491428A2F98ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);

        /* Rounds 4-7 */
        MSG1 = _mm_loadu_si128((const __m128i*) (data+16));
        MSG1 = _mm_shuffle_epi8(MSG1, MASK);
        MSG = _mm_add_epi32(MSG1, _mm_set_epi64x(0xAB1C5ED5923F82A4ULL, 0x59F111F13956C25BULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG0 = _mm_sha256msg1_epu32(MSG0, MSG1);

        /* Rounds 8-11 */
        MSG2 = _mm_loadu_si128((const __m128i*) (data+32));
        MSG2 = _mm_shuffle_epi8(MSG2, MASK);
        MSG = _mm_add_epi32(MSG2, _mm_set_epi64x(0x550C7DC3243185BEULL, 0x12835B01D807AA98ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG1 = _mm_sha256msg1_epu32(MSG1, MSG2);

        /* Rounds 12-15 */
        MSG3 = _mm_loadu_si128((const __m128i*) (data+48));
        MSG3 = _mm_shuffle_epi8(MSG3, MASK);
        MSG = _mm_add_epi32(MSG3, _mm_set_epi64x(0xC19BF1749BDC06A7ULL, 0x80DEB1FE72BE5D74ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG3, MSG2, 4);
        MSG0 = _mm_add_epi32(MSG0, TMP);
        MSG0 = _mm_sha256msg2_epu32(MSG0, MSG3);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG2 = _mm_sha256msg1_epu32(MSG2, MSG3);

        /* Rounds 16-19 */
        MSG = _mm_add_epi32(MSG0, _mm_set_epi64x(0x240CA1CC0FC19DC6ULL, 0xEFBE4786E49B69C1ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG0, MSG3, 4);
        MSG1 = _mm_add_epi32(MSG1, TMP);
        MSG1 = _mm_sha256msg2_epu32(MSG1, MSG0);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG3 = _mm_sha256msg1_epu32(MSG3, MSG0);

        /* Rounds 20-23 */
        MSG = _mm_add_epi32(MSG1, _mm_set_epi64x(0x76F988DA5CB0A9DCULL, 0x4A7484AA2DE92C6FULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG1, MSG0, 4);
        MSG2 = _mm_add_epi32(MSG2, TMP);
        MSG2 = _mm_sha256msg2_epu32(MSG2, MSG1);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG0 = _mm_sha256msg1_epu32(MSG0, MSG1);

        /* Rounds 24-27 */
        MSG = _mm_add_epi32(MSG2, _mm_set_epi64x(0xBF597FC7B00327C8ULL, 0xA831C66D983E5152ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG2, MSG1, 4);
        MSG3 = _mm_add_epi32(MSG3, TMP);
        MSG3 = _mm_sha256msg2_epu32(MSG3, MSG2);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG1 = _mm_sha256msg1_epu32(MSG1, MSG2);

        /* Rounds 28-31 */
        MSG = _mm_add_epi32(MSG3, _mm_set_epi64x(0x1429296706CA6351ULL,  0xD5A79147C6E00BF3ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG3, MSG2, 4);
        MSG0 = _mm_add_epi32(MSG0, TMP);
        MSG0 = _mm_sha256msg2_epu32(MSG0, MSG3);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG2 = _mm_sha256msg1_epu32(MSG2, MSG3);

        /* Rounds 32-35 */
        MSG = _mm_add_epi32(MSG0, _mm_set_epi64x(0x53380D134D2C6DFCULL, 0x2E1B213827B70A85ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG0, MSG3, 4);
        MSG1 = _mm_add_epi32(MSG1, TMP);
        MSG1 = _mm_sha256msg2_epu32(MSG1, MSG0);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG3 = _mm_sha256msg1_epu32(MSG3, MSG0);

        /* Rounds 36-39 */
        MSG = _mm_add_epi32(MSG1, _mm_set_epi64x(0x92722C8581C2C92EULL, 0x766A0ABB650A7354ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG1, MSG0, 4);
        MSG2 = _mm_add_epi32(MSG2, TMP);
        MSG2 = _mm_sha256msg2_epu32(MSG2, MSG1);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG0 = _mm_sha256msg1_epu32(MSG0, MSG1);

        /* Rounds 40-43 */
        MSG = _mm_add_epi32(MSG2, _mm_set_epi64x(0xC76C51A3C24B8B70ULL, 0xA81A664BA2BFE8A1ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG2, MSG1, 4);
        MSG3 = _mm_add_epi32(MSG3, TMP);
        MSG3 = _mm_sha256msg2_epu32(MSG3, MSG2);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG1 = _mm_sha256msg1_epu32(MSG1, MSG2);

        /* Rounds 44-47 */
        MSG = _mm_add_epi32(MSG3, _mm_set_epi64x(0x106AA070F40E3585ULL, 0xD6990624D192E819ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG3, MSG2, 4);
        MSG0 = _mm_add_epi32(MSG0, TMP);
        MSG0 = _mm_sha256msg2_epu32(MSG0, MSG3);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG2 = _mm_sha256msg1_epu32(MSG2, MSG3);

        /* Rounds 48-51 */
        MSG = _mm_add_epi32(MSG0, _mm_set_epi64x(0x34B0BCB52748774CULL, 0x1E376C0819A4C116ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG0, MSG3, 4);
        MSG1 = _mm_add_epi32(MSG1, TMP);
        MSG1 = _mm_sha256msg2_epu32(MSG1, MSG0);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);
        MSG3 = _mm_sha256msg1_epu32(MSG3, MSG0);

        /* Rounds 52-55 */
        MSG = _mm_add_epi32(MSG1, _mm_set_epi64x(0x682E6FF35B9CCA4FULL, 0x4ED8AA4A391C0CB3ULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG1, MSG0, 4);
        MSG2 = _mm_add_epi32(MSG2, TMP);
        MSG2 = _mm_sha256msg2_epu32(MSG2, MSG1);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);

        /* Rounds 56-59 */
        MSG = _mm_add_epi32(MSG2, _mm_set_epi64x(0x8CC7020884C87814ULL, 0x78A5636F748F82EEULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        TMP = _mm_alignr_epi8(MSG2, MSG1, 4);
        MSG3 = _mm_add_epi32(MSG3, TMP);
        MSG3 = _mm_sha256msg2_epu32(MSG3, MSG2);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);

        /* Rounds 60-63 */
        MSG = _mm_add_epi32(MSG3, _mm_set_epi64x(0xC67178F2BEF9A3F7ULL, 0xA4506CEB90BEFFFAULL));
        STATE1 = _mm_sha256rnds2_epu32(STATE1, STATE0, MSG);
        MSG = _mm_shuffle_epi32(MSG, 0x0E);
        STATE0 = _mm_sha256rnds2_epu32(STATE0, STATE1, MSG);

        /* Combine state  */
        STATE0 = _mm_add_epi32(STATE0, ABEF_SAVE);
        STATE1 = _mm_add_epi32(STATE1, CDGH_SAVE);

        data += 64;
        length -= 64;
    }

    TMP = _mm_shuffle_epi32(STATE0, 0x1B);       /* FEBA */
    STATE1 = _mm_shuffle_epi32(STATE1, 0xB1);    /* DCHG */
    STATE0 = _mm_blend_epi16(TMP, STATE1, 0xF0); /* DCBA */
    STATE1 = _mm_alignr_epi8(STATE1, TMP, 8);    /* ABEF */

    /* Save state */
    _mm_storeu_si128((__m128i*) &state[0], STATE0);
    _mm_storeu_si128((__m128i*) &state[4], STATE1);
}

////////////////////////////////////////////////////////////////////////////////


// How long does it take to insert a number into a PowerSumAccumulator?
template <typename T_NUM_BITS, unsigned long NUM_BYTES>
void benchmark_insertion(
    std::size_t num_packets,
    std::size_t num_trials
) {

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<T_NUM_BITS> dist(
        std::numeric_limits<T_NUM_BITS>::min(),
        std::numeric_limits<T_NUM_BITS>::max()
    );

    // Allocate buffer for benchmark durations.
    std::vector<uint32_t> durations;

    for (std::size_t i = 0; i < num_trials + 1; ++i) {

        // Generate <num_packets> + 10 random numbers.
        std::vector<T_NUM_BITS> numbers;
        for (std::size_t j = 0; j < num_packets + 10; ++j) {
            numbers.push_back(dist(gen));
        }

        // Initialize a SHA256 hasher and buffer for the hash and count.
        unsigned char value[NUM_BYTES];
        uint16_t count = 0;
        uint32_t state[8] = {
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
        };

        // Warm up the instruction cache by inserting a few numbers.
        for (std::size_t i = num_packets; i < num_packets + 10; ++i) {
            std::memcpy(value, &numbers[i], NUM_BYTES);
            sha256_process_x86(state, value, NUM_BYTES);
        }

        // Insert a bunch of random numbers into the accumulator.
        begin_timer();
        for (std::size_t j = 0; j < num_packets; ++j) {
            std::memcpy(value, &numbers[i], NUM_BYTES);
            sha256_process_x86(state, value, NUM_BYTES);
            count++;
        }
        do_not_discard(state);
        end_timer();

        if (i > 0) {
            auto duration = print_timer("Insert " + std::to_string(num_packets)
                        + " numbers into Strawman2 (" +
                        std::string(TYPE_NAME<T_NUM_BITS>) + ")");
            durations.push_back(duration);
        }
    }
    print_summary(durations);
}


void run_insertion_benchmark(
    std::size_t num_packets,
    std::size_t num_bits_id,
    std::size_t num_trials
) {
    if (num_bits_id == 16) {
        benchmark_insertion<std::uint16_t, 2>(num_packets, num_trials);
    } else if (num_bits_id == 32) {
        benchmark_insertion<std::uint32_t, 4>(num_packets, num_trials);
    } else if (num_bits_id == 64) {
        benchmark_insertion<std::uint64_t, 8>(num_packets, num_trials);
    } else {
        std::cout << "ERROR: <num_bits_id> must be 16, 32, or 64" << std::endl;
    }
}


////////////////////////////////////////////////////////////////////////////////

#define NUM_SUBSETS_LIMIT 10000

std::size_t choose(std::size_t n, std::size_t k) {
    if (k == 0) return 1;
    return (n * choose(n - 1, k - 1)) / k;
}

// How long does it take to compute the set-theoretic difference between two
// PowerSumAccumulators, assuming one is a subset of the other?
template <typename T_NUM_BITS, unsigned long NUM_BYTES>
void benchmark_decode(
    std::size_t num_packets,
    std::size_t num_drop,
    std::size_t num_trials
) {

    // Initialize C++ PRNG.
    std::random_device rd;
    std::mt19937_64 gen(rd());
    std::uniform_int_distribution<T_NUM_BITS> dist(
        std::numeric_limits<T_NUM_BITS>::min(),
        std::numeric_limits<T_NUM_BITS>::max()
    );

    // Allocate buffer for benchmark durations.
    std::vector<uint32_t> durations;

    // Calculate the number of subsets.
    std::size_t num_subsets = choose(num_packets, num_drop);

    for (std::size_t i = 0; i < num_trials + 1; ++i) {
        // Generate 1000 random numbers.
        std::vector<T_NUM_BITS> numbers;
        for (std::size_t j = 0; j < num_packets; ++j) {
            numbers.push_back(dist(gen));
        }

        // Construct the SHA256 hash.
        unsigned char value[NUM_BYTES];
        uint32_t state[8] = {
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
        };
        for (std::size_t j = 0; j < num_packets - num_drop; ++j) {
            std::memcpy(value, &numbers[i], NUM_BYTES);
            sha256_process_x86(state, value, NUM_BYTES);
        }

        begin_timer();
        if (num_drop > 0) {
            // Do this num_subsets / 2 times to calculate the expected time.
            // But actually just calculate how many can be done in 30 seconds
            // and extrapolate.
            std::size_t num_hashes_to_calculate =
                std::min((std::size_t)NUM_SUBSETS_LIMIT, num_subsets / 2);
            for (std::size_t i = 0; i < num_hashes_to_calculate; i++) {
                // For every subset of size "num_packets - num_drop"
                // Calculate the SHA256 hash
                uint32_t state[8] = {
                    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
                };
                // We're really just measuring a lower bound of the time to
                // compute any SHA256 hash with this number of elements
                for (std::size_t j = 0; j < num_packets - num_drop; ++j) {
                    std::memcpy(value, &numbers[i], NUM_BYTES);
                    sha256_process_x86(state, value, NUM_BYTES);
                }
                do_not_discard(state);
            }
        }
        end_timer();

        if (i > 0) {
            auto duration =
                print_timer("Decode time (" + std::string(TYPE_NAME<T_NUM_BITS>) +
                        ", num_packets = " + std::to_string(num_packets) +
                        ", dropped = " + std::to_string(num_drop) + ")");
            durations.push_back(duration);
        }
    }
    uint32_t avg = print_summary(durations);
    if (num_subsets / 2 > NUM_SUBSETS_LIMIT) {
        std::cout << "Only calculated " << std::to_string(NUM_SUBSETS_LIMIT)
                  << " hashes, expected " << std::to_string(num_subsets / 2)
                  << " extrapolating -> ";
        avg /= NUM_SUBSETS_LIMIT;
        avg *= num_subsets / 2;
        // std::cout << avg * num_subsets / 2 / NUM_SUBSETS_LIMIT / 1'000'000'000
        std::cout << avg * num_subsets / NUM_SUBSETS_LIMIT / 2'000'000'000
                  << " s" << std::endl;
    }
}

void run_decode_benchmark(
    std::size_t num_packets,
    std::size_t num_bits_id,
    std::size_t num_drop,
    std::size_t num_trials
) {
    if (num_bits_id == 16) {
        benchmark_decode<std::uint16_t, 2>(num_packets, num_drop, num_trials);
    } else if (num_bits_id == 32) {
        benchmark_decode<std::uint32_t, 4>(num_packets, num_drop, num_trials);
    } else if (num_bits_id == 64) {
        benchmark_decode<std::uint64_t, 8>(num_packets, num_drop, num_trials);
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
    bool benchmark_insertion = false;
    bool benchmark_decode = false;
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
        } else if (std::string(argv[i]) == "--insertion") {
            benchmark_insertion = true;
        } else if (std::string(argv[i]) == "--decode") {
            benchmark_decode = true;
        }
    }

    if ((benchmark_insertion ^ benchmark_decode) && !help) {
        if (benchmark_insertion) {
            run_insertion_benchmark(
                num_packets,
                num_bits_id,
                num_trials
            );
        } else if (benchmark_decode) {
            run_decode_benchmark(
                num_packets,
                num_bits_id,
                num_drop,
                num_trials
            );
        }
    } else {
        std::cout << "Usage: " << argv[0] << " [-n <num_packets>] "
                  << "[-b <num_bits_id>] " << "[--dropped <num_drop>] "
                  << "[--trials <num_trials>] [--insertion] [--decode]"
                  << std::endl;
    }

    return 0;
}
