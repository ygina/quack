use std::time::{Instant, Duration};
use clap::{Parser, ValueEnum};
use rand::Rng;

mod modint;
mod accumulator;
mod evaluator;

use modint::ModularInteger;
use accumulator::PowerSumAccumulator;
use evaluator::MonicPolynomialEvaluator;

fn print_summary(d: Vec<Duration>) {
    let size = d.len() as u32;
    let avg = if d.is_empty() {
        Duration::new(0, 0)
    } else {
        d.into_iter().sum::<Duration>() / size
    };
    println!("SUMMARY: num_trials = {}, avg = {:?}", size, avg);
}


// // A trick from the Google Benchmark library: this prevents dead code
// // elimination on a variable that is calculated but never used.
// template <typename T>
// void do_not_discard(const T &value) {
//     asm volatile("" : : "r,m"(value) : "memory");
// }


// template <typename T>
// static constexpr const char *TYPE_NAME = "UNKNOWN";
// template <> constexpr const char *TYPE_NAME<std::uint16_t> = "16-bit integers";
// template <> constexpr const char *TYPE_NAME<std::uint32_t> = "32-bit integers";
// template <> constexpr const char *TYPE_NAME<std::uint64_t> = "64-bit integers";


// ////////////////////////////////////////////////////////////////////////////////


// #define MAX_POWER 50
// static const auto power_tables_16 = gen_power_tables_16<
//     UINT16_C(65'521)>(MAX_POWER);


// // How long does it take to insert a number into a PowerSumAccumulator?
// template <std::uint16_t MODULUS>
// void benchmark_construct_16(
//     usize size,
//     usize num_packets,
//     usize num_drop,
//     usize num_trials
// ) {

//     // Initialize C++ PRNG.
//     std::random_device rd;
//     std::mt19937_64 gen(rd());
//     std::uniform_int_distribution<std::uint16_t> dist(
//         std::numeric_limits<std::uint16_t>::min(),
//         std::numeric_limits<std::uint16_t>::max()
//     );

//     // Allocate buffer for benchmark durations.
//     std::vector<uint32_t> durations;

//     for (usize i = 0; i < num_trials + 1; ++i) {

//         // Generate <num_packets> + 10 random numbers.
//         std::vector<std::uint16_t> numbers;
//         // for (usize j = 0; j < num_packets + 10; ++j)
//         for (usize j = 0; j < num_packets; ++j)
//             numbers.push_back(dist(gen));

//         // Construct two empty PowerSumAccumulators.
//         PowerSumAccumulator<std::uint16_t, uint32_t, MODULUS> acc1(size);
//         PowerSumAccumulator<std::uint16_t, uint32_t, MODULUS> acc2(size);

//         // Warm up the instruction cache by inserting a few numbers.
//         for (usize i = num_packets; i < num_packets + 10; ++i) {
//             acc1.insert(numbers[i]);
//         }
//         for (usize i = num_packets; i < num_packets + 10; ++i) {
//             acc2.insert(numbers[i]);
//         }

//         // Insert a bunch of random numbers into the accumulator.
//         begin_timer();
//         for (usize j = 0; j < num_packets; ++j)
//             acc1.insert(power_tables_16, MAX_POWER, numbers[j]);
//         for (usize j = 0; j < num_packets - num_drop; ++j)
//             acc2.insert(power_tables_16, MAX_POWER, numbers[j]);
//         do_not_discard(acc1);
//         do_not_discard(acc2);
//         end_timer();

//         if (i > 0) {
//             auto duration = print_timer("Insert " + std::to_string(num_packets)
//                         + " numbers into 2 PowerSumAccumulators (" +
//                         std::string(TYPE_NAME<std::uint16_t>) + ", threshold = " +
//                         std::to_string(size) + ")");
//             durations.push_back(duration);
//         }
//     }
//     print_summary(durations);
// }


// template <std::uint32_t MODULUS>
// void benchmark_construct_24(
//     usize size,
//     usize num_packets,
//     usize num_drop,
//     usize num_trials
// ) {
//     static const auto power_tables_24 = gen_power_tables_24<
//         UINT32_C(16'777'049)>(MAX_POWER);

//     // Initialize C++ PRNG.
//     std::random_device rd;
//     std::mt19937_64 gen(rd());
//     std::uniform_int_distribution<std::uint32_t> dist(
//         std::numeric_limits<std::uint32_t>::min(),
//         16777216 - 1
//     );

//     // Allocate buffer for benchmark durations.
//     std::vector<uint32_t> durations;

//     for (usize i = 0; i < num_trials + 1; ++i) {

//         // Generate <num_packets> + 10 random numbers.
//         std::vector<std::uint32_t> numbers;
//         // for (usize j = 0; j < num_packets + 10; ++j)
//         for (usize j = 0; j < num_packets; ++j)
//             numbers.push_back(dist(gen));

//         // Construct two empty PowerSumAccumulators.
//         PowerSumAccumulator<std::uint32_t, uint64_t, MODULUS> acc1(size);
//         PowerSumAccumulator<std::uint32_t, uint64_t, MODULUS> acc2(size);

//         // Warm up the instruction cache by inserting a few numbers.
//         for (usize i = num_packets; i < num_packets + 10; ++i) {
//             acc1.insert(numbers[i]);
//         }
//         for (usize i = num_packets; i < num_packets + 10; ++i) {
//             acc2.insert(numbers[i]);
//         }

//         // Insert a bunch of random numbers into the accumulator.
//         begin_timer();
//         for (usize j = 0; j < num_packets; ++j)
//             acc1.insert(power_tables_24, MAX_POWER, numbers[j]);
//         for (usize j = 0; j < num_packets - num_drop; ++j)
//             acc2.insert(power_tables_24, MAX_POWER, numbers[j]);
//         do_not_discard(acc1);
//         do_not_discard(acc2);
//         end_timer();

//         if (i > 0) {
//             auto duration = print_timer("Insert " + std::to_string(num_packets)
//                         + " numbers into 2 PowerSumAccumulators (" +
//                         std::string(TYPE_NAME<std::uint32_t>) + ", threshold = " +
//                         std::to_string(size) + ")");
//             durations.push_back(duration);
//         }
//     }
//     print_summary(durations);
// }


// template <typename T_NARROW, typename T_WIDE, T_NARROW MAX_VALUE, T_NARROW
// MODULUS>
// <std::uint32_t, std::uint64_t,
//             std::numeric_limits<uint32_t>::max(), UINT32_C(4'294'967'291)>
fn benchmark_construct(
    size: usize,
    num_packets: usize,
    num_drop: usize,
    num_trials: usize,
) {
    let mut rng = rand::thread_rng();

    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        let numbers: Vec<u32> =
            (0..(num_packets + 10)).map(|_| rng.gen()).collect();

        // Construct two empty PowerSumAccumulators.
        let mut acc1 = PowerSumAccumulator::new(size);
        let mut acc2 = PowerSumAccumulator::new(size);

        // Warm up the instruction cache by inserting a few numbers.
        for i in num_packets..(num_packets + 10) {
            acc1.insert(numbers[i]);
        }
        for i in num_packets..(num_packets + 10) {
            acc2.insert(numbers[i]);
        }

        // Insert a bunch of random numbers into the accumulator.
        let t1 = Instant::now();
        for j in 0..num_packets {
            acc1.insert(numbers[j]);
        }
        for j in 0..(num_packets - num_drop) {
            acc2.insert(numbers[j]);
        }
        let t2 = Instant::now();

        if i > 0 {
            let duration = t2 - t1;
            println!("Insert {} numbers into 2 PowerSumAccumulators (u32, \
                threshold = {}): {:?}", num_packets, size, duration);
            durations.push(duration);
        }
    }
    print_summary(durations);
}


fn run_construct_benchmark(
    use_tables: bool,
    threshold: usize,
    num_packets: usize,
    num_bits_id: usize,
    num_drop: usize,
    num_trials: usize,
) {
//     if (num_bits_id == 16 && use_tables) {
//         benchmark_construct_16<UINT16_C(65'521)>(
//             threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 16 && !use_tables) {
//         benchmark_construct<std::uint16_t, std::uint32_t, UINT16_C(65'535),
//             UINT16_C(65'521)>(threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 24 && use_tables) {
//         benchmark_construct_24<UINT32_C(16'777'049)>(
//             threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 24 && !use_tables) {
//         benchmark_construct<std::uint32_t, std::uint64_t, UINT32_C(16'777'215),
//             UINT32_C(16'777'049)>(threshold, num_packets, num_drop, num_trials);
    if num_bits_id == 32 {
        benchmark_construct(threshold, num_packets, num_drop, num_trials);
//         benchmark_construct<std::uint32_t, std::uint64_t,
//             std::numeric_limits<uint32_t>::max(), UINT32_C(4'294'967'291)>
//             (threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 64) {
//         benchmark_construct<std::uint64_t, __uint128_t,
//             std::numeric_limits<uint64_t>::max(),
//             UINT64_C(18'446'744'073'709'551'557)>(threshold, num_packets, num_drop, num_trials);
    } else {
        eprintln!("ERROR: <num_bits_id> must be 16, 32, or 64");
    }
}


// ////////////////////////////////////////////////////////////////////////////////


// // How long does it take to compute the set-theoretic difference between two
// // PowerSumAccumulators, assuming one is a subset of the other?
// template <typename T_NARROW, typename T_WIDE, T_NARROW MAX_VALUE, T_NARROW
// MODULUS> void
fn benchmark_decode(
    size: usize,
    num_packets: usize,
    num_drop: usize,
    num_trials: usize,
) {
    let mut rng = rand::thread_rng();

    // Allocate buffer for benchmark durations.
    let mut durations: Vec<Duration> = vec![];

    for i in 0..(num_trials + 1) {
        // Allocate variable for counting false positives.
        let mut fp = 0;

        // Generate 1000 random numbers.
        let numbers: Vec<u32> =
            (0..num_packets).map(|_| rng.gen()).collect();

        // Construct two empty PowerSumAccumulators.
        let mut acc1 = PowerSumAccumulator::new(size);
        let mut acc2 = PowerSumAccumulator::new(size);

        // Insert all random numbers into the first accumulator.
        for j in 0..num_packets {
            acc1.insert(numbers[j]);
        }

        // Insert all but num_drop random numbers into the second accumulator.
        for j in 0..(num_packets - num_drop) {
            acc2.insert(numbers[j]);
        }

        // Pre-allocate buffer for polynomial coefficients.
        let mut coeffs = (0..num_drop).map(|_| ModularInteger::zero()).collect();

        // Allocate buffer for missing packets.
        let mut dropped: Vec<u32> = vec![];

        let t1 = Instant::now();
        if num_drop > 0 {
            acc1 -= acc2;
            acc1.to_polynomial_coefficients(&mut coeffs);
            for j in 0..(num_packets - num_drop) {
                let value = MonicPolynomialEvaluator::eval(&coeffs, numbers[j]);
                if value.is_zero() {
                    fp += 1;
                }
            }
            for j in (num_packets - num_drop)..num_packets {
                let value = MonicPolynomialEvaluator::eval(&coeffs, numbers[j]);
                assert!(value.is_zero());
                dropped.push(numbers[j]);
            }
        }
        // do_not_discard(dropped);
        let t2 = Instant::now();

        if i > 0 {
            let duration = t2 - t1;
            println!("Decode time (u32, threshold = {}, num_packets={}, \
                false_positives = {}, dropped = {}): {:?}", size, num_packets,
                fp, num_drop, duration);
            durations.push(duration);
        }
    }

    print_summary(durations);
}


// template <std::uint16_t MODULUS>
// void benchmark_decode_16(
//     usize size,
//     usize num_packets,
//     usize num_drop,
//     usize num_trials
// ) {

//     // Initialize C++ PRNG.
//     std::random_device rd;
//     std::mt19937_64 gen(rd());
//     std::uniform_int_distribution<std::uint16_t> dist(
//         std::numeric_limits<std::uint16_t>::min(),
//         std::numeric_limits<std::uint16_t>::max()
//     );

//     // Allocate buffer for benchmark durations.
//     std::vector<uint32_t> durations;

//     using evaluator = MonicPolynomialEvaluator<std::uint16_t, std::uint32_t, MODULUS>;

//     for (usize i = 0; i < num_trials + 1; ++i) {
//         // Allocate variable for counting false positives.
//         usize fp = 0;

//         // Generate 1000 random numbers.
//         std::vector<std::uint16_t> numbers;
//         for (usize j = 0; j < num_packets; ++j) {
//             numbers.push_back(dist(gen));
//         }

//         // Construct two empty PowerSumAccumulators.
//         PowerSumAccumulator<std::uint16_t, std::uint32_t, MODULUS> acc_1(size);
//         PowerSumAccumulator<std::uint16_t, std::uint32_t, MODULUS> acc_2(size);

//         // Insert all random numbers into the first accumulator.
//         for (usize j = 0; j < num_packets; ++j) {
//             acc_1.insert(numbers[j]);
//         }

//         // Insert all but num_drop random numbers into the second accumulator.
//         for (usize j = 0; j < num_packets - num_drop; ++j) {
//             acc_2.insert(numbers[j]);
//         }

//         // Pre-allocate buffer for polynomial coefficients.
//         std::vector<ModularInteger<std::uint16_t, std::uint32_t, MODULUS>> coeffs(num_drop);

//         begin_timer();
//         if (num_drop > 0) {
//             acc_1 -= acc_2;
//             acc_1.to_polynomial_coefficients(coeffs);
//             for (usize j = 0; j < num_packets - num_drop; ++j) {
//                 const auto value = evaluator::eval(power_tables_16,
//                     MAX_POWER, coeffs, numbers[j]);
//                 if (!value) fp++;
//                 do_not_discard(value);
//             }
//             for (usize j = num_packets - num_drop; j < num_packets; ++j) {
//                 const auto value = evaluator::eval(power_tables_16,
//                     MAX_POWER, coeffs, numbers[j]);
//                 assert(!value);
//                 do_not_discard(value);
//             }
//         }
//         end_timer();

//         if (i > 0) {
//             auto duration =
//                 print_timer("Decode time (" + std::string(TYPE_NAME<std::uint16_t>) +
//                         ", threshold = " + std::to_string(size) +
//                         ", num_packets = " + std::to_string(num_packets) +
//                         ", false_positives = " + std::to_string(fp) +
//                         ", dropped = " + std::to_string(num_drop) + ")");
//             durations.push_back(duration);
//         }
//     }

//     print_summary(durations);
// }


// template <std::uint32_t MODULUS>
// void benchmark_decode_24(
//     usize size,
//     usize num_packets,
//     usize num_drop,
//     usize num_trials
// ) {
//     static const auto power_tables_24 = gen_power_tables_24<
//         UINT32_C(16'777'049)>(MAX_POWER);

//     // Initialize C++ PRNG.
//     std::random_device rd;
//     std::mt19937_64 gen(rd());
//     std::uniform_int_distribution<std::uint32_t> dist(
//         std::numeric_limits<std::uint32_t>::min(),
//         16777216 - 1
//     );

//     // Allocate buffer for benchmark durations.
//     std::vector<uint32_t> durations;

//     using evaluator = MonicPolynomialEvaluator<std::uint32_t, std::uint64_t, MODULUS>;

//     for (usize i = 0; i < num_trials + 1; ++i) {
//         // Allocate variable for counting false positives.
//         usize fp = 0;

//         // Generate 1000 random numbers.
//         std::vector<std::uint32_t> numbers;
//         for (usize j = 0; j < num_packets; ++j) {
//             numbers.push_back(dist(gen));
//         }

//         // Construct two empty PowerSumAccumulators.
//         PowerSumAccumulator<std::uint32_t, std::uint64_t, MODULUS> acc_1(size);
//         PowerSumAccumulator<std::uint32_t, std::uint64_t, MODULUS> acc_2(size);

//         // Insert all random numbers into the first accumulator.
//         for (usize j = 0; j < num_packets; ++j) {
//             acc_1.insert(numbers[j]);
//         }

//         // Insert all but num_drop random numbers into the second accumulator.
//         for (usize j = 0; j < num_packets - num_drop; ++j) {
//             acc_2.insert(numbers[j]);
//         }

//         // Pre-allocate buffer for polynomial coefficients.
//         std::vector<ModularInteger<std::uint32_t, std::uint64_t, MODULUS>> coeffs(num_drop);

//         // Allocate buffer for missing packets.
//         std::vector<std::uint32_t> dropped;

//         begin_timer();
//         if (num_drop > 0) {
//             acc_1 -= acc_2;
//             acc_1.to_polynomial_coefficients(coeffs);
//             for (usize j = 0; j < num_packets - num_drop; ++j) {
//                 const auto value = evaluator::eval(
//                    power_tables_24, MAX_POWER, coeffs, numbers[j]);
//                 if (!value) fp++;
//                 do_not_discard(value);
//             }
//             for (usize j = num_packets - num_drop; j < num_packets; ++j) {
//                 const auto value = evaluator::eval(
//                    power_tables_24, MAX_POWER, coeffs, numbers[j]);
//                 assert(!value);
//                 do_not_discard(value);
//                 dropped.push_back(numbers[j]);
//             }
//         }
//         do_not_discard(dropped);
//         end_timer();

//         if (i > 0) {
//             auto duration =
//                 print_timer("Decode time (" + std::string(TYPE_NAME<std::uint32_t>) +
//                         ", threshold = " + std::to_string(size) +
//                         ", num_packets = " + std::to_string(num_packets) +
//                         ", false_positives = " + std::to_string(fp) +
//                         ", dropped = " + std::to_string(num_drop) + ")");
//             durations.push_back(duration);
//         }
//     }

//     print_summary(durations);
// }


fn run_decode_benchmark(
    use_tables: bool,
    threshold: usize,
    num_packets: usize,
    num_bits_id: usize,
    num_drop: usize,
    num_trials: usize,
) {
//     if (num_bits_id == 16 && use_tables) {
//         benchmark_decode_16<UINT16_C(65'521)>(
//             threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 16 && !use_tables) {
//         benchmark_decode<std::uint16_t, std::uint32_t, UINT16_C(65'535),
//             UINT16_C(65'521)>(threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 24 && use_tables) {
//         benchmark_decode_24<UINT32_C(16'777'049)>(
//             threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 24 && !use_tables) {
//         benchmark_decode<std::uint32_t, std::uint64_t, UINT32_C(16'777'215),
//             UINT32_C(16'777'049)>(threshold, num_packets, num_drop, num_trials);
    if num_bits_id == 32 {
//         benchmark_decode<std::uint32_t, std::uint64_t,
//             std::numeric_limits<uint32_t>::max(), UINT32_C(4'294'967'291)>
//             (threshold, num_packets, num_drop, num_trials);
        benchmark_decode(threshold, num_packets, num_drop, num_trials);
//     } else if (num_bits_id == 64) {
//         benchmark_decode<std::uint64_t, __uint128_t,
//             std::numeric_limits<uint64_t>::max(),
//             UINT64_C(18'446'744'073'709'551'557)>(threshold, num_packets, num_drop, num_trials);
    } else {
        println!("ERROR: <num_bits_id> must be 16, 32, or 64");
    }
}


// ////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, ValueEnum)]
enum BenchmarkType {
    Construct,
    Decode,
}

#[derive(Parser)]
struct Cli {
    // Type of benchmark.
    #[arg(value_enum)]
    benchmark: BenchmarkType,
    // The threshold number of dropped packets.
    #[arg(short = 't', default_value_t = 20)]
    threshold: usize,
    // Number of sent packets.
    #[arg(short = 'n', default_value_t = 1000)]
    num_packets: usize,
    // Number of identifier bits.
    #[arg(short = 'b', default_value_t = 32)]
    num_bits_id: usize,
    // Number of dropped packets.
    #[arg(long = "dropped", default_value_t = 20)]
    num_drop: usize,
    // Number of trials.
    #[arg(long = "trials", default_value_t = 10)]
    num_trials: usize,
    // Whether to use power tables.
    #[arg(long = "use-tables")]
    use_tables: bool,
}


fn main() {
    let args = Cli::parse();
    match args.benchmark {
        BenchmarkType::Construct => {
            run_construct_benchmark(
                args.use_tables,
                args.threshold,
                args.num_packets,
                args.num_bits_id,
                args.num_drop,
                args.num_trials,
            )
        }
        BenchmarkType::Decode => {
            run_decode_benchmark(
                args.use_tables,
                args.threshold,
                args.num_packets,
                args.num_bits_id,
                args.num_drop,
                args.num_trials,
            )
        }
    }
}
