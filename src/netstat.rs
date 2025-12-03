use bitflags::bitflags;
use std::{collections::HashMap, env, fmt::Display, io, net::SocketAddr, str::FromStr};
use tabled::Tabled;

use crate::netstat::linux::LinuxNetStat;

mod linux;

pub type PID = u32;

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct Protocol: u8 {
        const TCP = 1 << 0;
        // const TCPv6 = 1 << 1;
        const UDP = 1 << 2;
        // const UDPv6 = 1 << 3;
    }
}

impl FromStr for Protocol {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "tcp" => Protocol::TCP,
            // "tcp6" => Connections::TCPv6,
            "udp" => Protocol::UDP,
            // "udp6" => Connections::UDPv6,
            _ => return Err("Invalid Connection Type"),
        })
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Protocol::TCP => "tcp",
                // Connections::TCPv6 => "tcp6",
                Protocol::UDP => "udp",
                // Connections::UDPv6 => "udp6",
                _ => unreachable!(),
            }
        )
    }
}

#[derive(Debug, Tabled)]
pub struct NetStatEntry {
    pub exe: String,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub proto: Protocol,
}

pub trait NetStat {
    fn get_ports(&self, connections: Protocol) -> io::Result<HashMap<PID, NetStatEntry>>;
}

pub fn get_netstat_impl() -> Box<dyn NetStat> {
    let instance = match env::consts::OS {
        "linux" => LinuxNetStat::new(),
        _ => todo!("Not Yet Supported"),
    };
    Box::new(instance)
}

pub fn truncate_path(s: &str, limit: usize) -> String {
    let start = s[s.len() - limit..]
        .find("/")
        .map(|i| s.len() - limit + i + 1)
        .unwrap_or(s.len() - limit);
    s[start..].to_owned()
}
