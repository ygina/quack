#[macro_use]
extern crate log;

use std::fs;
use std::path::Path;
use std::io::Write;

use clap::{Arg, Command};
use pcap::{Capture, Device};

fn pcap_listen(
    mut f: fs::File,
    bytes: usize,
    timeout: i32,
) {
    let device = Device::lookup().unwrap();
    info!("listening on {:?}", device);
    let mut cap = Capture::from_device(device).unwrap()
        .promisc(true)
        .timeout(timeout)
        .snaplen(bytes as i32)
        .open().unwrap();

    let mut n: usize = 0;
    while let Ok(packet) = cap.next() {
        let len = std::cmp::min(packet.data.len(), bytes as usize);
        f.write_all(&packet.data[..len]).unwrap();
        if len < bytes {
            f.write_all(&vec![0; bytes - len]).unwrap();
        }
        f.flush().unwrap();
        n += 1;
        if n % 1000 == 0 {
            debug!("processed {} packets", n);
        }
    }
}

fn main() {
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
        .arg(Arg::new("filename")
            .help("File to write to. Fails if the file already exists \
                and the overwrite option is not passed.")
            .short('f')
            .long("filename")
            .takes_value(true)
            .default_value("router.txt"))
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
    let f = fs::File::create(path).unwrap();
    pcap_listen(f, bytes, timeout);
}
