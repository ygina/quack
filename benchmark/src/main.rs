#[macro_use]
extern crate log;

pub mod generator;

use clap::{Arg, Command};
use accumulator::Accumulator;
use accumulator::{CBFAccumulator, NaiveAccumulator, PowerSumAccumulator};
use generator::LoadGenerator;

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
        .arg(Arg::new("accumulator")
            .help("")
            .short('a')
            .long("accumulator")
            .takes_value(true)
            .possible_value("naive")
            .possible_value("cbf")
            .possible_value("power_sum")
            .required(true))
        .get_matches();

    let num_logged: usize = matches.value_of("num-logged").unwrap()
        .parse().unwrap();
    let p_dropped: f32 = matches.value_of("p-dropped").unwrap()
        .parse().unwrap();
    let malicious: bool = matches.is_present("malicious");
    debug!("num_logged = {}", num_logged);
    debug!("p_dropped = {}", p_dropped);
    debug!("malicious = {}", malicious);

    let mut accumulator: Box<dyn Accumulator> = {
        match matches.value_of("accumulator").unwrap() {
            "naive" => Box::new(NaiveAccumulator::new()),
            "cbf" => Box::new(CBFAccumulator::new()),
            "power_sum" => Box::new(PowerSumAccumulator::new()),
            _ => unreachable!(),
        }
    };
    let mut g = LoadGenerator::new(num_logged, p_dropped, malicious);
    while let Some(elem) = g.next() {
        accumulator.process(elem);
    }
    debug!("dropped {}/{} elements", g.num_dropped, g.num_logged);

    // Validate the log against the accumulator.
    let valid = accumulator.validate(&g.log);
    info!("valid? {} (expected {})", valid, !malicious);
}
