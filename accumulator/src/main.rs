#[macro_use]
extern crate log;

use std::fs::{OpenOptions, File};
use std::net::TcpListener;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use clap::{Arg, Command};
use accumulator::*;

use pcap_parser::*;

fn write_data(f: &mut File, bytes: usize, data: &[u8]) {
    let len = std::cmp::min(data.len(), bytes);
    if len < bytes {
        f.write_all(&vec![0; bytes - len]).unwrap();
    }
    f.write_all(&data[..len]).unwrap();
}

async fn pcap_listen_mock(
    log: Option<&str>,
    bytes: usize,
    accumulator: Arc<Mutex<Box<dyn Accumulator + Send>>>,
) {
    let packets = vec![
        vec![125; bytes],
        vec![50; bytes - 1],
        vec![26; bytes + 1],
    ];
    let mut log = log.map(|filename| {
        let path = std::path::Path::new(filename);
        if !path.exists() {
            File::create(filename).unwrap();
        }
        OpenOptions::new()
            .append(true)
            .open(filename)
            .unwrap()
    });
    let mut accumulator = accumulator.lock().unwrap();
    for data in packets {
        let len = std::cmp::min(data.len(), bytes as usize);
        let elem = &data[..len];
        if let Some(f) = log.as_mut() {
            write_data(f, bytes, &data[..len]);
        }
        accumulator.process(&elem);
    }
    drop(accumulator);
}

async fn pcap_listen(
    log: Option<&str>,
    bytes: usize,
    accumulator: Arc<Mutex<Box<dyn Accumulator + Send>>>,
) {
    use std::process::{Command, Stdio};
    let mut child = {
        let tcpdump = Command::new("tcpdump")
            .arg("-B")
            .arg("1024")
            .arg("-i")
            .arg("eth1")
            .arg("-Q")
            .arg("in")
            .arg("-w")
            .arg("/dev/stdout")
            .arg("-s")
            .arg(format!("{}", 14 + bytes))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(log_filename) = log {
            Command::new("tee")
                .arg(log_filename)
                .stdin(tcpdump.stdout.unwrap())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap()
        } else {
            tcpdump
        }
    };

    let stdout = child.stdout.as_mut().unwrap();

    let mut reader = create_reader(65536, stdout).unwrap();
    let mut n: usize = 0;
    loop {
        // TODO: This isn't perfect, because tcpdump is set up to buffer so reader.next() could
        // theoretically block for an arbitrarily long period of time.
        match reader.next() {
            Ok((offset, block)) => {
                match block {
                    PcapBlockOwned::Legacy(block) => {
                        if block.data.len() < 14 {
                            warn!("TOO SMALL");
                            continue;
                        }
                        let hi = std::cmp::min(14 + bytes, block.data.len());
                        let mut elem = block.data[14..hi].to_vec();
                        if elem.len() < bytes {
                            elem.append(&mut vec![0; bytes - elem.len()]);
                        }
                        let hi = std::cmp::min(block.data.len(), 14 + bytes as usize);
                        let elem = &block.data[14..hi];
                        // NOTE: many of these elements are not unique
                        // TODO: probably slow to put a lock around each packet.
                        // Maybe we can buffer and batch.
                        let mut accumulator = accumulator.lock().unwrap();
                        accumulator.process(&elem);
                        drop(accumulator);
                        n += 1;
                        trace!("processed {} packets block {:?} offset={}", n, block.data, offset);
                        if n % 1000 == 0 {
                            trace!("processed {} packets", n);
                        }
                    },
                    PcapBlockOwned::NG(block) => {
                        debug!("ignoring NG({:?}) offset={}", block, offset);
                    },
                    PcapBlockOwned::LegacyHeader(_) => {},
                }
                reader.consume(offset);
            },
            Err(PcapError::Eof) => {
                debug!("eof");
                break;
            },
            Err(PcapError::Incomplete) => {
                trace!("reader buffer size may be too small, or input file may be truncated.");
                reader.refill().unwrap();
            },
            Err(e) => error!("error while reading: {:?}", e),
        }
    }
}

async fn tcp_listen(
    accumulator: Arc<Mutex<Box<dyn Accumulator + Send>>>,
    port: u32,
) {
    info!("listening on port {}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut reset = [0; 1];
        stream.read(&mut reset).unwrap();
        let mut accumulator = accumulator.lock().unwrap();
        let bytes = accumulator.to_bytes();
        info!("reset byte = {}", reset[0]);
        if reset[0] != 0 {
            accumulator.reset();
        }
        drop(accumulator);
        info!("sending {} bytes to {:?}", bytes.len(), stream.peer_addr());
        stream.write_all(&bytes).unwrap();
        stream.flush().unwrap();
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let matches = Command::new("accumulator")
        .arg(Arg::new("mock")
            .help("Write mock data.")
            .long("mock"))
        .arg(Arg::new("log")
            .help("Whether to log data received in PCAP. FOR DEBUGGING \
                PURPOSES ONLY. Normally, the device running the accumulator \
                would not have enough space to maintain these logs. Writes \
                to the given filename (suggested: accum.pcap).")
            .long("log")
            .takes_value(true))
        .arg(Arg::new("port")
            .help("TCP port to listen on. Returns the serialized digest to \
                any connection on this port")
            .short('p')
            .long("port")
            .takes_value(true)
            .default_value("7878"))
        .arg(Arg::new("bytes")
            .help("Number of bytes to record from each packet. Default is \
                40 bytes, enough to capture an IPv6 header.")
            .short('b')
            .long("bytes")
            .takes_value(true)
            .default_value("40"))
        .arg(Arg::new("threshold")
            .help("Threshold number of log packets for the IBLT \
                and power sum accumulators.")
            .short('t')
            .long("threshold")
            .takes_value(true)
            .default_value("10000"))
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

    let bytes: usize = matches.value_of("bytes").unwrap().parse().unwrap();
    let port: u32 = matches.value_of("port").unwrap().parse().unwrap();
    let log = matches.value_of("log");
    let accumulator: Box<dyn Accumulator + Send> = {
        let threshold: usize = matches.value_of("threshold").unwrap()
            .parse().unwrap();
        match matches.value_of("accumulator").unwrap() {
            "naive" => Box::new(NaiveAccumulator::new(None)),
            "iblt" => Box::new(IBLTAccumulator::new(threshold, None)),
            "power_sum" => Box::new(PowerSumAccumulator::new(threshold, None)),
            _ => unreachable!(),
        }
    };
    let lock = Arc::new(Mutex::new(accumulator));
    let lock_clone = Arc::clone(&lock);
    let join = tokio::spawn(async move {
        tcp_listen(lock_clone, port).await;
    });
    if matches.is_present("mock") {
        pcap_listen_mock(log, bytes, lock).await;
    } else {
        pcap_listen(log, bytes, lock).await;
    }
    join.await.unwrap();
}
