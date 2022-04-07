#[macro_use]
extern crate log;

use std::net::TcpListener;
use std::io::Write;

use bincode;
use clap::{Arg, Command};
use num_bigint::BigUint;
use pcap::{Capture, Device};
use accumulator::*;

const PACKET_LEN: i32 = 128 / 8;

async fn pcap_listen(mut accumulator: Box<dyn Accumulator>, timeout: i32) {
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

async fn tcp_listen(port: u32) {
    info!("listening on port {}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let digest = b"1234";
        // let bytes = bincode::serialize(&digest).unwrap();
        let bytes = digest;
        info!("sending {} bytes to {:?}", bytes.len(), stream.peer_addr());
        stream.write(bytes).unwrap();
        stream.flush().unwrap();
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let matches = Command::new("benchmark")
        .arg(Arg::new("timeout")
            .help("Read timeout for the capture, in ms. The library uses 0 \
                by default, blocking indefinitely, but causes the capture \
                to hang in MacOS.")
            .long("timeout")
            .takes_value(true)
            .default_value("10000000"))
        .arg(Arg::new("port")
            .help("TCP port to listen on. Returns the serialized digest to \
                any connection on this port")
            .short('p')
            .long("port")
            .takes_value(true)
            .default_value("7878"))
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
    let port: u32 = matches.value_of("port").unwrap().parse().unwrap();
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

    tokio::spawn(async move {
        tcp_listen(port).await;
    });
    pcap_listen(accumulator, timeout).await;
}
