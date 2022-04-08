#[macro_use]
extern crate log;

use num_bigint::BigUint;
use accumulator::Accumulator;

/// Call the accumulator's TCP service and read the bytes.
/// Assume we know which type of accumulator it is using.
/// TODO: SSH into Pi and call the TCP service from there since
/// the TCP port shouldn't be externally exposed.
fn get_accumulator() -> Box<dyn Accumulator> {
    unimplemented!()
}

/// Read the file that contains the router logs.
/// TODO: SFTP logs from router.
fn get_router_logs() -> Vec<BigUint> {
    unimplemented!()
}

fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let accumulator = get_accumulator();
    let router_logs = get_router_logs();
    if accumulator.validate(&router_logs) {
        info!("valid router");
    } else {
        warn!("invalid router");
    }
}
