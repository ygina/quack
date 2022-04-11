#[macro_use]
extern crate log;

use std::net::TcpStream;
use std::io::Read;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::io::Cursor;

use bincode;
use ssh2::Session;
use clap::{Arg, Command};
use num_bigint::BigUint;
use accumulator::*;

use pcap_parser::*;
// use pcap_parser::traits::PcapReaderIterator;
// use std::fs::File;

/// Connect to the SSH server and assert the session is authenticated.
fn establish_ssh_session(
    addr: &str,
    username: &str,
    private_key_file: &str,
) -> Session {
    debug!("establishing ssh connection to {}", addr);
    let tcp = TcpStream::connect(addr).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_pubkey_file(
        username,
        None,
        Path::new(private_key_file),
        None,
    ).unwrap();
    assert!(sess.authenticated());
    sess
}

/// Call the accumulator's TCP service and read the bytes.
/// Assume we know which type of accumulator it is using.
/// TODO: SSH into Pi and call the TCP service from there since
/// the TCP port shouldn't be externally exposed.
fn get_accumulator(
    ssh: Option<Vec<&str>>,
    port: u32,
    ty: &str,
) -> Box<dyn Accumulator> {
    let mut buf = Vec::new();
    if let Some(ssh) = ssh {
        let sess = establish_ssh_session(ssh[0], ssh[1], ssh[2]);
        let mut channel = sess.channel_session().unwrap();
        let cmd = format!("cat /dev/null | nc -v 127.0.0.1 {}", port);
        channel.exec(&cmd).unwrap();
        channel.read_to_end(&mut buf).unwrap();
        channel.wait_close().unwrap();
        let exit_status = channel.exit_status().unwrap();
        if exit_status != 0 {
            error!("channel exit status: {}", exit_status);
            panic!("error retrieving accumulator digest");
        }
    } else {
        let address = format!("127.0.0.1:{}", port);
        let mut stream = TcpStream::connect(address).unwrap();
        stream.read_to_end(&mut buf).unwrap();
    };
    info!("accumulator size = {} bytes", buf.len());
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
/// - `ssh`: address and port to SSH into, if provided
/// - `filename`: name of the file, if remote make sure to specify full path
/// - `nbytes`: number of bytes per packet
/// TODO: SFTP logs from router.
fn get_router_logs(
    ssh: Option<Vec<&str>>,
    filename: &str,
    nbytes: usize,
) -> Vec<BigUint> {
    let data: Vec<u8> = if let Some(ssh) = ssh {
        let sess = establish_ssh_session(ssh[0], ssh[1], ssh[2]);
        let (mut f, stat) = sess.scp_recv(Path::new(filename)).unwrap();
        debug!("remote file size: {}", stat.size());
        let mut data: Vec<u8> = Vec::new();
        f.read_to_end(&mut data).unwrap();

        // Close the channel and wait for the whole content to be tranferred
        f.send_eof().unwrap();
        f.wait_eof().unwrap();
        f.close().unwrap();
        f.wait_close().unwrap();
        data
    } else {
        if !std::path::Path::new(filename).exists() {
            panic!("file does not exist: {}", filename);
        }
        debug!("reading router logs from {}", filename);
        std::fs::read(filename).unwrap()
    };

    // https://docs.rs/pcap-parser/latest/pcap_parser/struct.PcapNGReader.html
    debug!("parsing pcap format {} bytes", data.len());
    let mut reader = create_reader(65536, Cursor::new(data)).unwrap();
    let mut res = Vec::new();
    let mut n = 0;
    let mut maybe_truncated = false;
    loop {
        match reader.next() {
            Ok((offset, block)) => {
                maybe_truncated = false;
                match block {
                    PcapBlockOwned::Legacy(block) => {
                        n += 1;
                        res.push(BigUint::from_bytes_be(&block.data[14..(14 + nbytes)]));
                        if n % 1000 == 0 {
                            debug!("processed {} packets", n);
                        }
                    },
                    PcapBlockOwned::NG(block) => {
                        debug!("ignoring NG({:?}) offset={}", block, offset);
                    },
                    PcapBlockOwned::LegacyHeader(block) => {
                        debug!("ignoring LegacyHeader({:?}) offset={}", block, offset);
                    },
                }
                reader.consume(offset);
            },
            Err(PcapError::Eof) => {
                debug!("reached eof");
                break;
            },
            Err(PcapError::Incomplete) => {
                if maybe_truncated {
                    debug!("input file may be truncated");
                    break;
                }
                trace!("reader buffer size is too small");
                maybe_truncated = true;
                reader.refill().unwrap();
            },
            Err(e) => error!("error while reading: {:?}", e),
        }
    }
    debug!("parsed {} packets", res.len());
    res
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
    let mut is_subset = true;
    for (k, v2) in m2.iter() {
        if let Some(v1) = m1.get(k) {
            if v1 < v2 {
                is_subset = false;
                break;
            }
        } else {
            is_subset = false;
            break;
        }
    }
    if is_subset {
        info!("m2 is a subset of m1");
    } else {
        warn!("m2 is not a subset of m1");
    }

    if m1.len() == m2.len() {
        debug!("both maps have {} entries", m1.len());
    } else {
        debug!("# entries differs: {} != {}", m1.len(), m2.len());
    }
    let m1_keys = m1.keys().collect::<HashSet<_>>();
    let m2_keys = m2.keys().collect::<HashSet<_>>();
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
    let router_logs = get_router_logs(None, router_filename, bytes);
    let router_logs_map = to_map(&router_logs);
    for i in 0..std::cmp::min(10, router_logs.len()) {
        println!("{:X}", router_logs[i]);
    }
    info!("accumulator logs:");
    let accumulator_logs = get_router_logs(None, acc_filename, bytes);
    let accumulator_logs_map = to_map(&accumulator_logs);
    for i in 0..std::cmp::min(10, accumulator_logs.len()) {
        println!("{:X}", accumulator_logs[i]);
    }
    compare_maps(router_logs_map, accumulator_logs_map);
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();

    let matches = Command::new("verifier")
        .arg(Arg::new("check-acc-logs")
            .help("Whether to check accumulator logs against router logs. \
                FOR DEBUGGING ONLY. (suggested: accum.pcap)")
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
            .default_value("router.pcap"))
        .arg(Arg::new("bytes")
            .help("Number of bytes recorded from each packet. Default is \
                128 bits = 16 bytes.")
            .short('b')
            .long("bytes")
            .takes_value(true)
            .default_value("16"))
        .arg(Arg::new("router-ssh")
            .help("Address of the router to SSH into (if not local) i.e. \
                `openwrt.lan:22`, the username, and the path to the private \
                key file.")
            .long("router-ssh")
            .takes_value(true)
            .multiple_values(true)
            .number_of_values(3)
            .value_names(&["address", "username", "private_key_file"]))
        .arg(Arg::new("accumulator-ssh")
            .help("Address of the accumulator to SSH into (if not local) i.e. \
                `openwrt.lan:22`, the username, and the path to the private \
                key file.")
            .long("accumulator-ssh")
            .takes_value(true)
            .multiple_values(true)
            .number_of_values(3)
            .value_names(&["address", "username", "private_key_file"]))
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

    let accumulator = get_accumulator(
        matches.values_of("accumulator-ssh").map(|ssh| ssh.collect()),
        port,
        accumulator_type,
    );
    let router_logs = get_router_logs(
        matches.values_of("router-ssh").map(|ssh| ssh.collect()),
        filename,
        bytes,
    );
    info!("{}/{} packets received", accumulator.total(), router_logs.len());
    assert!(accumulator.total() <= router_logs.len());
    if accumulator.validate(&router_logs) {
        info!("valid router");
    } else {
        warn!("invalid router");
    }
}
