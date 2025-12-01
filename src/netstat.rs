use bitflags::bitflags;
use std::{collections::HashMap, env, io, net::SocketAddr};

use crate::netstat::linux::LinuxNetStat;

mod linux;

pub type PID = u32;

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct Connections: u8 {
        const TCP = 1 << 0;
        const TCPv6 = 1 << 1;
        const UDP = 1 << 2;
        const UDPv6 = 1 << 3;
    }
}

#[derive(Debug)]
pub struct NetStatEntry {
    exe: String,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    proto: Connections,
}

pub trait NetStat {
    fn get_ports(&self, connections: Connections) -> io::Result<HashMap<PID, NetStatEntry>>;
}

pub fn get_netstat_impl() -> Box<dyn NetStat> {
    let instance = match env::consts::OS {
        "linux" => LinuxNetStat::new(),
        _ => todo!("Not Yet Supported"),
    };
    Box::new(instance)
}
