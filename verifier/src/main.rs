#[macro_use]
extern crate log;

use std::net::TcpStream;
use std::io::Read;

use bincode;
use clap::{Arg, Command};
use num_bigint::BigUint;
use accumulator::*;

/// Call the accumulator's TCP service and read the bytes.
/// Assume we know which type of accumulator it is using.
/// TODO: SSH into Pi and call the TCP service from there since
/// the TCP port shouldn't be externally exposed.
fn get_accumulator(address: &str, ty: &str) -> Box<dyn Accumulator> {
    let mut stream = TcpStream::connect(address).unwrap();
    let mut buf: Vec<u8> = vec![];
    let n = stream.read_to_end(&mut buf).unwrap();
    info!("accumulator size = {} bytes", n);
    info!("accumulator type = {}", ty);
    match ty {
        "naive" => Box::new(bincode::deserialize::<NaiveAccumulator>(&buf).unwrap()),
        "cbf" => Box::new(bincode::deserialize::<CBFAccumulator>(&buf).unwrap()),
        "iblt" => Box::new(bincode::deserialize::<IBLTAccumulator>(&buf).unwrap()),
        "power_sum" => Box::new(bincode::deserialize::<PowerSumAccumulator>(&buf).unwrap()),
        _ => unreachable!(),
    }
}

/// Read the file that contains the router logs.
/// - `filename`: name of the file
/// - `nbytes`: number of bytes per packet
/// TODO: SFTP logs from router.
fn get_router_logs(filename: &str, nbytes: usize) -> Vec<BigUint> {
    if !std::path::Path::new(filename).exists() {
        panic!("file does not exist: {}", filename);
    }
    let data = std::fs::read(filename).unwrap();
    let n_packets = data.len() / nbytes;
    (0..n_packets)
        .map(|i| ((i * nbytes), (i+1) * nbytes))
        .map(|(start, end)| BigUint::from_bytes_be(&data[start..end]))
        .collect()
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let matches = Command::new("verifier")
        .arg(Arg::new("port")
            .help("Port of the accumulator's TCP service.")
            .short('p')
            .long("port")
            .takes_value(true)
            .default_value("7878"))
        .arg(Arg::new("filename")
            .help("File to read router logs.")
            .short('f')
            .long("filename")
            .takes_value(true)
            .default_value("router.txt"))
        .arg(Arg::new("bytes")
            .help("Number of bytes recorded from each packet. Default is \
                128 bits = 16 bytes.")
            .short('b')
            .long("bytes")
            .takes_value(true)
            .default_value("16"))
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

    let port: u32 = matches.value_of("port").unwrap().parse().unwrap();
    let filename = matches.value_of("filename").unwrap();
    let bytes: usize = matches.value_of("bytes").unwrap().parse().unwrap();
    let accumulator_type = matches.value_of("accumulator").unwrap();

    let address = format!("127.0.0.1:{}", port);
    let accumulator = get_accumulator(&address, accumulator_type);
    let router_logs = get_router_logs(filename, bytes);
    info!("{}/{} packets received", accumulator.total(), router_logs.len());
    assert!(accumulator.total() < router_logs.len());
    if accumulator.validate(&router_logs) {
        info!("valid router");
    } else {
        warn!("invalid router");
    }
}
