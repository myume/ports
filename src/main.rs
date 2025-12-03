use std::io;

use clap::{Parser, Subcommand, command};
use ports::netstat::{NetStatEntry, Protocol, get_netstat_impl, truncate_path};
use tabled::Table;

mod tui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The protocol to query
    #[arg(short, long, default_value_t = Protocol::TCP)]
    proto: Protocol,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Open an interactive TUI
    Tui,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let netstat = get_netstat_impl();

    match args.command {
        Some(Commands::Tui) => {
            let mut tui = tui::Tui::new(netstat, args.proto);
            let mut terminal = ratatui::init();
            let app_result = tui.run(&mut terminal);
            ratatui::restore();
            app_result
        }
        None => {
            let ports: Vec<NetStatEntry> = netstat
                .get_ports(&args.proto)?
                .into_iter()
                .map(|mut port| {
                    port.exe = format!("...{}", truncate_path(&port.exe, 32));
                    port
                })
                .collect();

            let table = Table::new(ports);
            println!("{table}");
            Ok(())
        }
    }
}
