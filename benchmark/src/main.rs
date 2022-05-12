#[macro_use]
extern crate log;

pub mod generator;

use std::time::{Instant, Duration};
use clap::{Arg, Command};
use accumulator::*;
use generator::LoadGenerator;

fn build_accumulator(
    g: &mut LoadGenerator,
    accumulator_ty: &str,
    threshold: usize,
    iblt_params: Option<Vec<&str>>,
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
                    threshold, bits_per_entry, cells_multiplier, num_hashes))
            } else {
                Box::new(IBLTAccumulator::new(threshold))
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
    info!("digest size = {} bytes", accumulator.to_bytes().len());
    accumulator
}

/// Validate the log against the accumulator.
fn validate(
    accumulator: Box<dyn Accumulator>,
    elems: &Vec<Vec<u8>>,
    malicious: bool,
) -> Result<Duration, ()> {
    let t1 = Instant::now();
    let valid = accumulator.validate(elems);
    let total = Instant::now() - t1;
    if valid == !malicious {
        info!("validation is correct ({}): {:?}", valid, total);
        Ok(total)
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
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
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

    let trials: usize = matches.value_of_t("trials").unwrap();
    let num_logged: usize = matches.value_of("num-logged").unwrap()
        .parse().unwrap();
    let p_dropped: f32 = matches.value_of("p-dropped").unwrap()
        .parse().unwrap();
    let malicious: bool = matches.is_present("malicious");
    debug!("num_logged = {}", num_logged);
    debug!("p_dropped = {}", p_dropped);
    debug!("malicious = {}", malicious);

    let accumulator_ty = matches.value_of("accumulator").unwrap();
    let threshold: usize = matches.value_of("threshold").unwrap()
        .parse().unwrap();
    let iblt_params: Option<Vec<&str>> = matches.values_of("iblt-params")
        .map(|values| values.collect());

    let mut results = vec![];
    let mut errors = 0;
    for _ in 0..trials {
        let mut g = LoadGenerator::new(num_logged, p_dropped, malicious);
        let acc = build_accumulator(&mut g, accumulator_ty, threshold,
            iblt_params.clone());
        if let Ok(result) = validate(acc, &g.log, malicious) {
            results.push(result);
        } else {
            errors += 1;
        }
    }
    info!("errors\tlogged\tp_drop\tmedian");
    info!("{}\t{}\t{}\t{:?}", errors, num_logged, p_dropped, median(results));
}
