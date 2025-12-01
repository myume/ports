use ports::netstat::{Connections, get_netstat_impl};

fn main() {
    let netstat = get_netstat_impl();

    if let Ok(port_mapping) = netstat.get_ports(Connections::TCP) {
        println!("{:#?}", port_mapping);
    }
}
