#[macro_use]
extern crate log;

use std::net::TcpStream;
use std::io::Read;
use std::collections::{HashMap, HashSet};

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

/// Logs seem to have many repeated entries.
/// Maps log values to number of occurrences.
fn to_map(logs: &Vec<BigUint>) -> HashMap<BigUint, usize> {
    let mut map: HashMap<BigUint, usize> = HashMap::new();
    for entry in logs {
        *map.entry(entry.clone()).or_insert(0) += 1;
    }
    map
}

/// Compares maps to each other. Metrics include number of entries, number of
/// shared keys, and counts of shared keys.
fn compare_maps(m1: HashMap<BigUint, usize>, m2: HashMap<BigUint, usize>) {
    if m1.len() == m2.len() {
        debug!("both maps have {} entries", m1.len());
    } else {
        debug!("# entries differs: {} != {}", m1.len(), m2.len());
    }
    let m1_keys = m1.keys().collect::<HashSet<_>>();
    let m2_keys = m1.keys().collect::<HashSet<_>>();
    let mut shared_keys: HashSet<BigUint> = HashSet::new();
    let mut m1_only: HashSet<BigUint> = HashSet::new();
    let mut m2_only: HashSet<BigUint> = HashSet::new();
    for &k in &m1_keys {
        if !m2_keys.contains(k) {
            m1_only.insert(k.clone());
        } else {
            shared_keys.insert(k.clone());
        }
    }
    for &k in &m2_keys {
        if !m1_keys.contains(k) {
            m2_only.insert(k.clone());
        } else {
            shared_keys.insert(k.clone());
        }
    }
    debug!("{} shared keys", shared_keys.len());
    debug!("{} keys in m1 only", m1_only.len());
    debug!("{} keys in m2 only", m2_only.len());
    for k in &shared_keys {
        let m1_v = m1.get(k).unwrap();
        let m2_v = m2.get(k).unwrap();
        if m1_v != m2_v {
            debug!("shared key {} values differ: {} != {}", k, m1_v, m2_v);
        }
    }
}

/// Check the accumulator logs against the router logs (DEBUGGING ONLY).
fn check_acc_logs(router_filename: &str, acc_filename: &str, bytes: usize) {
    info!("router logs:");
    let router_logs = get_router_logs(router_filename, bytes);
    let router_logs_map = to_map(&router_logs);
    for i in 0..std::cmp::min(10, router_logs.len()) {
        println!("{:?}", router_logs[i]);
    }
    info!("accumulator logs:");
    let accumulator_logs = get_router_logs(acc_filename, bytes);
    let accumulator_logs_map = to_map(&accumulator_logs);
    for i in 0..std::cmp::min(10, accumulator_logs.len()) {
        println!("{:?}", accumulator_logs[i]);
    }
    compare_maps(router_logs_map, accumulator_logs_map);
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let matches = Command::new("verifier")
        .arg(Arg::new("check-acc-logs")
            .help("Whether to check accumulator logs against router logs. \
                FOR DEBUGGING ONLY. (suggested: log.txt)")
            .long("check-acc-logs")
            .takes_value(true))
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

    if let Some(acc_filename) = matches.value_of("check-acc-logs") {
        check_acc_logs(filename, acc_filename, bytes)
    }

    let address = format!("127.0.0.1:{}", port);
    let accumulator = get_accumulator(&address, accumulator_type);
    let router_logs = get_router_logs(filename, bytes);
    info!("{}/{} packets received", accumulator.total(), router_logs.len());
    assert!(accumulator.total() <= router_logs.len());
    if accumulator.validate(&router_logs) {
        info!("valid router");
    } else {
        warn!("invalid router");
    }
}
