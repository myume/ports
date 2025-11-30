use ports::netstat::{Connections, get_netstat_impl};

fn main() {
    let netstat = get_netstat_impl();

    println!("{:?}", netstat.get_ports(Connections::TCP));
}
