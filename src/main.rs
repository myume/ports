use clap::{Parser, command};
use ports::netstat::{NetStatEntry, Protocol, get_netstat_impl, truncate_path};
use tabled::Table;

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
        Ok(mapping) => {
            let ports = mapping
                .into_values()
                .map(|mut port| {
                    port.exe = format!("...{}", truncate_path(&port.exe, 32));
                    port
                })
                .collect::<Vec<NetStatEntry>>();
            let table = Table::new(ports);
            println!("{table}");
        }
        Err(e) => eprintln!("Failed to get ports: {e}"),
    }
}
