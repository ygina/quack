use clap::Parser;
use quack::{Quack, IBLTQuackU32};

const NUM_ERRORS: [usize; 8] = [
    1, 2, 5, 10, 20, 50, 100, 200,
];

#[derive(Parser, Debug)]
struct Cli {
    /// Whether to run the decode benchmark, and num_errors if provided
    #[arg(long, default_value_t = 100000)]
    num_iters: usize,
    /// Number of errors to evaluate
    #[arg(long, value_delimiter=',')]
    num_errors: Option<Vec<usize>>,
}

struct BenchmarkResult {
    // Number of errors
    num_errors: usize,
    // Number of symbols to successfully decode
    num_symbols: Vec<usize>,
}

impl BenchmarkResult {
    fn new() -> Self {
        Self {
            num_errors: 0,
            num_symbols: vec![],
        }
    }

    fn print(&mut self) {
        self.num_symbols.sort();
        println!("BenchmarkIBLTMultiplier/d={:<4}{:8} {}",
            self.num_errors, self.num_symbols.len(),
            self.num_symbols.iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
            );
    }
}

fn one_iblt_decode(
    log: &Vec<u32>, num_symbols: usize,
) -> bool {
    let mut q = IBLTQuackU32::new(num_symbols);
    for &id in log {
        q.insert(id);
    }
    q.decode().is_some()
}

fn benchmark_iblt_multiplier(num_iters: usize, num_errors: usize) {
    // Initialize the result
    let mut result = BenchmarkResult::new();

    // Benchmark decoding
    let mut next_id = 0;
    let mut log = vec![0; num_errors];
    for _ in 0..num_iters {
        for i in 0..num_errors {
            next_id += 1;
            log[i] = next_id;
        }

        // Find the number of coded symbols needed to decode
        let mut lo = num_errors;
        let mut hi = 2 * lo;
        loop {
            if one_iblt_decode(&log, hi) {
                break;
            }
            lo *= 2;
            hi *= 2;
        }
        while lo < hi {
            let mid = (lo + hi) / 2;
            if one_iblt_decode(&log, mid) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }

        result.num_symbols.push(lo);
    }

    // Set other result fields
    result.num_errors = num_errors;
    result.print();
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    for num_errors in args.num_errors.unwrap_or(NUM_ERRORS.to_vec()) {
        benchmark_iblt_multiplier(args.num_iters, num_errors);
    }
}
