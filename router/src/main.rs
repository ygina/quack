#[macro_use]
extern crate log;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn write_data(f: &mut File, bytes: usize, data: &[u8]) {
    let len = std::cmp::min(data.len(), bytes);
    if len < bytes {
        f.write_all(&vec![0; bytes - len]).unwrap();
    }
    f.write_all(&data[..len]).unwrap();
}

fn pcap_listen_mock(
    f: &mut File,
    bytes: usize,
    _timeout: i32,
) {
    warn!("mock data - not an actual pcap file");
    let packets = vec![
        vec![125; bytes],
        vec![50; bytes - 1],
        vec![26; bytes + 1],
        vec![88; bytes],
    ];
    for data in packets {
        write_data(f, bytes, &data);
    }
}

fn pcap_listen(
    fname: &str,
    bytes: usize,
    timeout: i32,
) {
    use std::process::Command;
    use signal_child::{Signalable, signal};

    debug!("listening on tcpdump");
    let mut child = Command::new("tcpdump")
        .arg("-w")
        .arg(fname)
        .arg("-s")
        .arg(format!("{}", 14 + bytes))
        .spawn()
        .unwrap();

    // TODO: This seems to be dropping lots of packets at the end, call sigusr2 and give it some
    // extra time before signinting.
    // https://doc.rust-lang.org/stable/std/thread/fn.sleep.html
    use std::{thread, time};
    thread::sleep(time::Duration::from_millis(timeout.try_into().unwrap()));
    child.signal(signal::SIGUSR2).expect("Error interrupting child");
    child.signal(signal::SIGINT).expect("Error interrupting child");
    child.wait().ok();
    info!("exiting");
}

fn main() {
    use clap::{Arg, Command};
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let matches = Command::new("router")
        .arg(Arg::new("timeout")
            .help("Read timeout for the capture, in ms. The library uses 0 \
                by default, blocking indefinitely, but causes the capture \
                to hang in MacOS.")
            .short('t')
            .long("timeout")
            .takes_value(true)
            .default_value("10000000"))
        .arg(Arg::new("overwrite")
            .help("Overwrites the file if it already exists.")
            .long("overwrite"))
        .arg(Arg::new("mock")
            .help("Write mock data.")
            .long("mock"))
        .arg(Arg::new("filename")
            .help("File to write to. Fails if the file already exists \
                and the overwrite option is not passed.")
            .short('f')
            .long("filename")
            .takes_value(true)
            .default_value("router.pcap"))
        .arg(Arg::new("bytes")
            .help("Number of bytes to record from each packet. Default is \
                128 bits = 16 bytes.")
            .short('b')
            .long("bytes")
            .takes_value(true)
            .default_value("16"))
        .get_matches();

    let timeout: i32 = matches.value_of("timeout").unwrap().parse().unwrap();
    let bytes: usize = matches.value_of("bytes").unwrap().parse().unwrap();
    let filename = matches.value_of("filename").unwrap();
    let overwrite = matches.is_present("overwrite");

    let path = Path::new(filename);
    if path.exists() && !overwrite {
        warn!("cannot overwrite file: {:?}", path);
        return;
    }
    info!("writing router data to {}", filename);
    let mut f = fs::File::create(path).unwrap();
    if matches.is_present("mock") {
        pcap_listen_mock(&mut f, bytes, timeout);
    } else {
        drop(f);
        pcap_listen(&filename, bytes, timeout);
    }
}
