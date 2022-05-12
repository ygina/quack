#[macro_use]
extern crate log;

pub mod generator;

use std::time::{Instant, Duration};
use clap::{Arg, Command};
use accumulator::*;
use generator::{SeedGenerator, LoadGenerator};

fn build_accumulator(
    g: &mut LoadGenerator,
    accumulator_ty: &str,
    threshold: usize,
    iblt_params: Option<Vec<&str>>,
    seed: Option<u64>,
) -> Box<dyn Accumulator> {
    let mut accumulator: Box<dyn Accumulator> = {
        match accumulator_ty {
            "naive" => Box::new(NaiveAccumulator::new()),
            "iblt" => if let Some(params) = iblt_params {
                assert_eq!(params.len(), 3);
                let bits_per_entry: usize = params[0].parse().unwrap();
                let cells_multiplier: usize = params[1].parse().unwrap();
                let num_hashes: u32 = params[2].parse().unwrap();
                Box::new(IBLTAccumulator::new_with_params(
                    threshold, bits_per_entry, cells_multiplier, num_hashes,
                    seed))
            } else {
                Box::new(IBLTAccumulator::new(threshold, seed))
            },
            "power_sum" => Box::new(PowerSumAccumulator::new(threshold)),
            _ => unreachable!(),
        }
    };
    let t1 = Instant::now();
    while let Some(elem) = g.next() {
        accumulator.process(&elem);
    }
    let t2 = Instant::now();
    debug!(
        "dropped {}/{} elements: {:?}",
        g.num_dropped,
        g.num_logged,
        t2 - t1,
    );
    accumulator
}

/// Validate the log against the accumulator.
fn validate(
    accumulator: Box<dyn Accumulator>,
    elems: &Vec<Vec<u8>>,
    malicious: bool,
) -> Result<(Duration, ValidationResult), ()> {
    let t1 = Instant::now();
    let result = accumulator.validate(elems);
    let total = Instant::now() - t1;
    if result.is_valid() == !malicious {
        info!("validation is correct ({:?}): {:?}", result, total);
        Ok((total, result))
    } else {
        error!("validation failed, expected {}", !malicious);
        Err(())
    }
}

fn median(mut results: Vec<Duration>) -> Duration {
    results.sort();
    let mid = results.len() / 2;
    if results.len() & 1 == 0 {
        (results[mid] + results[mid + 1]) / 2
    } else {
        results[mid]
    }
}

fn main() {
    let matches = Command::new("benchmark")
        .arg(Arg::new("num-logged")
            .help("Number of logged packets.")
            .short('n')
            .long("num-logged")
            .takes_value(true)
            .default_value("100000"))
        .arg(Arg::new("p-dropped")
            .help("Probability that a logged packet is dropped.")
            .short('p')
            .long("p-dropped")
            .takes_value(true)
            .default_value("0.005"))
        .arg(Arg::new("debug-level")
            .help("Debug level.")
            .long("debug-level")
            .takes_value(true)
            .possible_value("trace")
            .possible_value("debug")
            .possible_value("info")
            .possible_value("warn")
            .possible_value("error")
            .possible_value("off")
            .default_value("debug"))
        .arg(Arg::new("malicious")
            .help("Whether the router sends a packet without logging. \
                The index of the malicious packet is randomly selected, \
                and the logged packet at that index is randomly set \
                and definitely not dropped.")
            .long("malicious"))
        .arg(Arg::new("threshold")
            .help("Threshold number of log packets for the CBF \
                and power sum accumulators.")
            .short('t')
            .long("threshold")
            .takes_value(true)
            .default_value("1000"))
        .arg(Arg::new("iblt-params")
            .help("IBLT parameters.")
            .long("iblt-params")
            .value_names(&["bits_per_entry", "cells_multiplier", "num_hashes"])
            .takes_value(true)
            .number_of_values(3))
        .arg(Arg::new("seed")
            .help("IBLT and load generator seed for reproducible results.")
            .long("seed")
            .takes_value(true))
        .arg(Arg::new("trials")
            .help("Number of trials to run. Reports the median.")
            .long("trials")
            .takes_value(true)
            .default_value("1"))
        .arg(Arg::new("accumulator")
            .help("")
            .short('a')
            .long("accumulator")
            .takes_value(true)
            .possible_value("naive")
            .possible_value("iblt")
            .possible_value("power_sum")
            .required(true))
        .get_matches();

    let debug_level = match matches.value_of("debug-level").unwrap() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        "off" => log::LevelFilter::Off,
        _ => unreachable!(),
    };
    env_logger::builder().filter_level(debug_level).init();
    let trials: usize = matches.value_of_t("trials").unwrap();
    let num_logged: usize = matches.value_of_t("num-logged").unwrap();
    let p_dropped: f32 = matches.value_of_t("p-dropped").unwrap();
    let malicious: bool = matches.is_present("malicious");
    debug!("num_logged = {}", num_logged);
    debug!("p_dropped = {}", p_dropped);
    debug!("malicious = {}", malicious);

    let accumulator_ty = matches.value_of("accumulator").unwrap();
    let threshold: usize = matches.value_of_t("threshold").unwrap();
    let iblt_params: Option<Vec<&str>> = matches.values_of("iblt-params")
        .map(|values| values.collect());
    let mut seed_generator = SeedGenerator::new(matches.value_of("seed")
        .map(|seed| seed.parse().unwrap()));

    let mut results = vec![];
    let mut errors = 0;
    let mut ilp = 0;
    for i in 0..trials {
        let seed = seed_generator.next();
        let mut g = LoadGenerator::new(seed, num_logged, p_dropped, malicious);
        let acc = build_accumulator(&mut g, accumulator_ty, threshold,
            iblt_params.clone(), seed.clone());
        if i == trials - 1 {
            warn!("digest size = {} bytes", acc.to_bytes().len());
        }
        if let Ok((duration, result)) = validate(acc, &g.log, malicious) {
            results.push(duration);
            if result == ValidationResult::IbltIlpValid
                || result == ValidationResult::IbltIlpInvalid {
                ilp += 1;
            }
        } else {
            errors += 1;
        }
    }
    warn!("trials\tilp\terrors\tlogged\tp_drop\tmedian");
    warn!("{}\t{}\t{}\t{}\t{}\t{:?}", trials, ilp, errors, num_logged,
       p_dropped, median(results));
}
