use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, BufRead, BufReader},
    net::IpAddr,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::netstat::{Connections, NetStat, PID};

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
    fn get_ports(&self, connections: Connections) -> io::Result<HashMap<PID, IpAddr>> {
        let pids = fs::read_dir(&self.proc_path)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_string_lossy().parse::<PID>().ok());

        let mut mapping = HashMap::new();
        for pid in pids {
            let pid_path = self.proc_path.join(pid.to_string());
            for connection in connections {
                let socket_filename = match connection {
                    Connections::TCP => "tcp",
                    Connections::TCPv6 => "tcp6",
                    Connections::UDP => "udp",
                    Connections::UDPv6 => "udp6",
                    _ => unreachable!(),
                };
                let socket_table_file = pid_path.join("net").join(socket_filename);

                get_ports_for_pid(&pid_path, &socket_table_file)
                    .unwrap_or_default()
                    .into_iter()
                    .for_each(|addr| {
                        mapping.insert(pid, addr);
                    });
            }
        }

        Ok(mapping)
    }
}

fn get_ports_for_pid(pid_path: &Path, socket_table_file: &Path) -> io::Result<Vec<IpAddr>> {
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

    let file = File::open(socket_table_file)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // read one line to remove the header
    reader.read_line(&mut line)?;
    line.clear();

    while let n = reader.read_line(&mut line)?
        && n > 0
    {
        println!("{} {line}", socket_table_file.display());
        line.clear();
    }
    todo!()
}
