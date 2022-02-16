use clap::{Arg, Command};

fn main() {
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
        .get_matches();

    let num_logged: usize = matches.value_of("num-logged").unwrap()
        .parse().unwrap();
    let p_dropped: f32 = matches.value_of("p-dropped").unwrap()
        .parse().unwrap();
    let malicious: bool = matches.is_present("malicious");
    println!("num_logged = {}", num_logged);
    println!("p_dropped = {}", p_dropped);
    println!("malicious = {}", malicious);
}
