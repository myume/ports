use clap::{Parser, command};
use ports::netstat::{Protocol, get_netstat_impl};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// List the current ports in use
    #[arg(short, long)]
    list: bool,

    #[arg(short, long, default_value_t = Protocol::TCP)]
    proto: Protocol,
}

fn main() {
    let args = Args::parse();

    let netstat = get_netstat_impl();

    match netstat.get_ports(args.proto) {
        Ok(mapping) => println!("{:#?}", mapping),
        Err(e) => eprintln!("Failed to get ports: {e}"),
    }
}
