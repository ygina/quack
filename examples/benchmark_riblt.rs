use std::time::{Instant, Duration};
use clap::{Parser, ValueEnum};
use quack::{PowerSumQuack, PowerSumQuackU32, Quack, IBLTQuackU32, QuackWrapper};

const NUM_SYMBOLS: [usize; 11] = [
    10, 20, 40, 80, 160, 320, 1000, 10000, 100000, 1000000, 10000000,
];

const NUM_ERRORS: [usize; 10] = [
    1, 2, 4, 8, 16, 20, 40, 80, 160, (u8::MAX - 1) as usize,
];

const TARGET_DURATION_MS: u64 = 100;

#[derive(Parser, Debug)]
struct Cli {
    /// Quack type
    #[arg(value_enum)]
    quack_ty: QuackType,
    /// Whether to run the encode benchmark
    #[arg(long)]
    encode: bool,
    /// Whether to run the decode benchmark
    #[arg(long)]
    decode: bool,
}

#[derive(ValueEnum, Debug, Copy, Clone)]
enum QuackType {
    PowerSum,
    IBLT,
}

trait BenchmarkResult {
    fn new(ty: QuackType) -> Self where Self: Sized;
    fn time(&self) -> Duration;
    fn print(&self);
}

struct EncodeBenchmarkResult {
    ty: QuackType,
    // Number of coded symbols
    num_symbols: usize,
    // Number of iterations
    num_iters: usize,
    // Total time
    time: Duration,
    // Total number of bytes of a serialized quack
    nbytes: usize,
}

impl BenchmarkResult for EncodeBenchmarkResult {
    fn new(ty: QuackType) -> Self {
        Self {
            ty,
            num_symbols: 0,
            num_iters: 0,
            time: Duration::from_secs(0),
            nbytes: 0,
        }
    }

    fn time(&self) -> Duration {
        self.time
    }

    fn print(&self) {
        println!("Benchmark{:?}Encode/m={:<10}{:8}{:>13}ns/op{:12} bytes",
            self.ty, self.num_symbols, self.num_iters,
            (self.time / (self.num_iters as u32)).as_nanos(),
            self.nbytes);
    }
}

struct DecodeBenchmarkResult {
    ty: QuackType,
    // Number of errors
    num_errors: usize,
    // Number of iterations
    num_iters: usize,
    // Total time
    time: Duration,
    // Number of successful iterations at 2x symbols/diff
    succ_2x: usize,
    // Number of successful iterations at 4x symbols/diff
    succ_4x: usize,
    // Total number of coded symbols processed to decode num_errors over
    // num_iters. Lower bound is 1 symbol/diff/iter. RIBLT paper says 1.35-1.75.
    symbols: usize,
}

impl BenchmarkResult for DecodeBenchmarkResult {
    fn new(ty: QuackType) -> Self {
        Self {
            ty,
            num_errors: 0,
            num_iters: 0,
            time: Duration::from_secs(0),
            succ_2x: 0,
            succ_4x: 0,
            symbols: 0,
        }
    }

    fn time(&self) -> Duration {
        self.time
    }

    fn print(&self) {
        println!("Benchmark{:?}Decode/d={:<10}{:8}{:>13?}ns/op{:10.3} succ@2x{:10.3} succ@4x{:10.3} symbols/diff",
            self.ty, self.num_errors, self.num_iters,
            (self.time / (self.num_iters as u32)).as_nanos(),
            (self.succ_2x as f64) / (self.num_iters as f64),
            (self.succ_4x as f64) / (self.num_iters as f64),
            (self.symbols as f64) / ((self.num_iters * self.num_errors) as f64));
    }
}

fn benchmark_encode(
    quack_ty: QuackType, num_symbols: usize, num_iters: usize,
) -> Box<dyn BenchmarkResult> {
    // Setup the benchmark
    let mut q = match quack_ty {
        QuackType::PowerSum => QuackWrapper::new(num_symbols, false),
        QuackType::IBLT => QuackWrapper::new(num_symbols, true),
    };

    // Benchmark encoding
    let t1 = Instant::now();
    for i in 0..num_iters {
        q.insert(i as u32);
    }
    let t2 = Instant::now();

    // Set the result
    let mut result = EncodeBenchmarkResult::new(quack_ty);
    result.num_symbols = num_symbols;
    result.num_iters = num_iters;
    result.time = t2 - t1;
    result.nbytes = q.serialize().len();
    Box::new(result)
}

fn benchmark_psum_decode(quack_ty: QuackType, num_errors: usize, num_iters: usize) -> Box<dyn BenchmarkResult> {
    // Initialize the result
    let mut result = DecodeBenchmarkResult::new(quack_ty);
    // Power sum decoding is actually proportional to num_shared, unlike RIBLT,
    // since we plug candidate ids into the polynomial. 49x is like 2% loss.
    let num_shared = 49 * num_errors;
    let num_symbols = num_errors;
    let num_ids = num_shared + num_errors;

    // Benchmark decoding
    let mut next_id = 0;
    let mut log = vec![0; num_ids];
    for _ in 0..num_iters {
        for i in 0..num_ids {
            next_id += 1;
            log[i] = next_id;
        }

        let mut q1 = PowerSumQuackU32::new(num_symbols);
        let mut q2 = PowerSumQuackU32::new(num_symbols);
        for i in 0..(num_errors + num_shared) {
            q1.insert(log[i]);
        }
        for i in 0..num_shared {
            q2.insert(log[i]);
        }

        let t1 = Instant::now();
        q1.sub_assign(&q2);
        let _ = q1.decode_with_log(log.as_slice());
        let t2 = Instant::now();
        result.time += t2 - t1;
    }

    // Set other result fields
    result.num_errors = num_errors;
    result.num_iters = num_iters;
    result.symbols = num_symbols * num_iters;
    result.succ_2x = num_iters;
    result.succ_4x = num_iters;
    Box::new(result)
}

fn one_iblt_decode(result: &mut DecodeBenchmarkResult, num_errors: usize,
                   num_shared: usize, log: &Vec<u32>, num_symbols: usize,
                   time: bool) -> bool
{
    let mut q1 = IBLTQuackU32::new(num_symbols);
    let mut q2 = IBLTQuackU32::new(num_symbols);
    for i in 0..(num_errors + num_shared) {
        q1.insert(log[i]);
    }
    for i in 0..num_shared {
        q2.insert(log[i]);
    }

    let t1 = Instant::now();
    q1.sub_assign(&q2);
    let succ = q1.decode().is_some();
    let t2 = Instant::now();
    if time {
        result.time += t2 - t1;
    }
    succ
}

fn benchmark_iblt_decode(
    quack_ty: QuackType, num_errors: usize, num_iters: usize,
) -> Box<dyn BenchmarkResult> {
    // Initialize the result
    let mut result = DecodeBenchmarkResult::new(quack_ty);
    let num_shared = num_errors; // same number of shared symbols as errors
    let num_ids = num_shared + num_errors;

    // Benchmark decoding
    let mut next_id = 0;
    let mut log = vec![0; num_ids];
    for _ in 0..num_iters {
        for i in 0..num_ids {
            next_id += 1;
            log[i] = next_id;
        }

        // Find the number of coded symbols needed to decode
        let mut lo = num_errors;
        let mut hi = 2 * lo;
        loop {
            if one_iblt_decode(&mut result, num_errors, num_shared, &log, hi, false) {
                break;
            }
            lo *= 2;
            hi *= 2;
        }
        while lo < hi {
            let mid = (lo + hi) / 2;
            if one_iblt_decode(&mut result, num_errors, num_shared, &log, mid, false) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }

        // Benchmark it
        one_iblt_decode(&mut result, num_errors, num_shared, &log, lo, true);
        result.symbols += lo;

        // See if the number of coded symbols exceeds a threshold
        if lo <= 2 * num_errors {
            result.succ_2x += 1;
        }
        if lo <= 4 * num_errors {
            result.succ_4x += 1;
        }
    }

    // Set other result fields
    result.num_errors = num_errors;
    result.num_iters = num_iters;
    Box::new(result)
}

fn benchmark(
    quack_ty: QuackType,
    func: fn(QuackType, usize, usize) -> Box<dyn BenchmarkResult>,
    params: &[usize],
) {
    let target = Duration::from_millis(TARGET_DURATION_MS);
    for &param in params {
        let mut num_iters = 1;
        loop {
            let result = func(quack_ty, param, num_iters);
            if result.time() > target || num_iters > 10_000_000 {
                result.print();
                break;
            }
            num_iters *= 2;
        }
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    match args.quack_ty {
        QuackType::PowerSum => {
            quack::global_config_set_max_power_sum_threshold(
                *NUM_SYMBOLS.last().unwrap());
            if args.encode {
                benchmark(args.quack_ty, benchmark_encode, NUM_SYMBOLS.as_slice());
            }
            if args.decode {
                benchmark(args.quack_ty, benchmark_psum_decode, NUM_ERRORS.as_slice());
            }
        }
        QuackType::IBLT => {
            if args.encode {
                benchmark(args.quack_ty, benchmark_encode, NUM_SYMBOLS.as_slice());
            }
            if args.decode {
                benchmark(args.quack_ty, benchmark_iblt_decode, NUM_ERRORS.as_slice());
            }
        }
    }
}
