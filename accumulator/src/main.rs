#[macro_use]
extern crate log;

use clap::{Arg, Command};
use num_bigint::BigUint;
use pcap::{Capture, Device};
use accumulator::*;

const PACKET_LEN: i32 = 128 / 8;

fn pcap_listen(mut accumulator: Box<dyn Accumulator>, timeout: i32) {
    let device = Device::lookup().unwrap();
    info!("listening on {:?}", device);
    let mut cap = Capture::from_device(device).unwrap()
        .promisc(true)
        .timeout(timeout)
        .snaplen(PACKET_LEN)
        .open().unwrap();

    let mut n: usize = 0;
    while let Ok(packet) = cap.next() {
        let len = std::cmp::min(packet.data.len(), PACKET_LEN as usize);
        let elem = BigUint::from_bytes_be(&packet.data[..len]);
        // NOTE: many of these elements are not unique
        accumulator.process(&elem);
        n += 1;
        if n % 1000 == 0 {
            debug!("processed {} packets", n);
        }
    }
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let matches = Command::new("benchmark")
        .arg(Arg::new("timeout")
            .help("Read timeout for the capture, in ms. The library uses 0 \
                by default, blocking indefinitely, but causes the capture \
                to hang in MacOS.")
            .short('t')
            .long("timeout")
            .takes_value(true)
            .default_value("10000000"))
        .arg(Arg::new("threshold")
            .help("Threshold number of log packets for the CBF \
                and power sum accumulators.")
            .short('t')
            .long("threshold")
            .takes_value(true)
            .default_value("1000"))
        .arg(Arg::new("accumulator")
            .help("")
            .short('a')
            .long("accumulator")
            .takes_value(true)
            .possible_value("naive")
            .possible_value("cbf")
            .possible_value("iblt")
            .possible_value("power_sum")
            .required(true))
        .get_matches();

    let timeout: i32 = matches.value_of("timeout").unwrap().parse().unwrap();
    let accumulator: Box<dyn Accumulator> = {
        let threshold: usize = matches.value_of("threshold").unwrap()
            .parse().unwrap();
        match matches.value_of("accumulator").unwrap() {
            "naive" => Box::new(NaiveAccumulator::new()),
            "cbf" => Box::new(CBFAccumulator::new(threshold)),
            "iblt" => Box::new(IBLTAccumulator::new(threshold)),
            "power_sum" => Box::new(PowerSumAccumulator::new(threshold)),
            _ => unreachable!(),
        }
    };

    pcap_listen(accumulator, timeout);
}
