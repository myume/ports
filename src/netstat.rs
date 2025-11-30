use std::{collections::HashMap, env, io, net::IpAddr};

use crate::netstat::linux::LinuxNetStat;

mod linux;

pub type PID = u32;

pub trait NetStat {
    fn get_ports(&self) -> io::Result<HashMap<PID, IpAddr>>;
}

pub fn get_netstat_impl() -> Box<dyn NetStat> {
    let instance = match env::consts::OS {
        "linux" => LinuxNetStat::new(),
        _ => todo!("Not Yet Supported"),
    };
    Box::new(instance)
}
