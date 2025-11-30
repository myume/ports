use std::{
    collections::{HashMap, HashSet},
    fs, io,
    net::IpAddr,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::netstat::{NetStat, PID};

pub struct LinuxNetStat {
    proc_path: PathBuf,
}

impl LinuxNetStat {
    pub fn new() -> Self {
        Self {
            proc_path: PathBuf::from("/proc"),
        }
    }
}

impl NetStat for LinuxNetStat {
    fn get_ports(&self) -> io::Result<HashMap<PID, IpAddr>> {
        let pids = fs::read_dir(&self.proc_path)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_string_lossy().parse::<PID>().ok());

        let mut mapping = HashMap::new();
        for pid in pids {
            get_ports_for_pid(&self.proc_path.join(pid.to_string()))
                .unwrap_or_default()
                .into_iter()
                .for_each(|addr| {
                    mapping.insert(pid, addr);
                });
        }

        Ok(mapping)
    }
}

fn get_ports_for_pid(pid_path: &Path) -> io::Result<Vec<IpAddr>> {
    let socket_regex = Regex::new(r"^socket:\[(?<inode>\d+)\]$").unwrap();
    let inodes: HashSet<String> = fs::read_dir(pid_path.join("fd"))?
        .filter_map(|fd| fd.ok())
        .filter_map(|fd| fs::read_link(fd.path()).ok())
        .filter_map(|link| {
            let haystack = link.to_string_lossy();
            let caps = socket_regex.captures(&haystack)?;
            Some(caps["inode"].to_owned())
        })
        .collect();

    Ok(vec![])
}
