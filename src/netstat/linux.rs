use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, BufRead, BufReader},
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
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
    fn get_ports(&self, connections: Connections) -> io::Result<HashMap<PID, SocketAddr>> {
        let pids = fs::read_dir(&self.proc_path)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().to_string_lossy().parse::<PID>().ok());

        let mut mapping = HashMap::new();
        for pid in pids {
            let pid_path = self.proc_path.join(pid.to_string());
            let Ok(inodes) = get_socket_inodes(&pid_path) else {
                continue;
            };
            for connection in connections {
                let socket_filename = match connection {
                    Connections::TCP => "tcp",
                    Connections::TCPv6 => "tcp6",
                    Connections::UDP => "udp",
                    Connections::UDPv6 => "udp6",
                    _ => unreachable!(),
                };
                let socket_table_file = pid_path.join("net").join(socket_filename);

                get_ports_for_pid(&socket_table_file, &inodes)
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

fn get_socket_inodes(pid_path: &Path) -> io::Result<HashSet<String>> {
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
    Ok(inodes)
}

fn get_ports_for_pid(
    socket_table_file: &Path,
    inodes: &HashSet<String>,
) -> io::Result<Vec<SocketAddr>> {
    let file = File::open(socket_table_file)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // read one line to remove the header
    reader.read_line(&mut line)?;
    line.clear();

    // sl local_address rem_address st tx_queue rx_queue tr tm->when retrnsmt uid timeout inode
    let table_line = Regex::new(
        r"(?x)
        ^\s*
        (\d*):
        \s*
        (?<local_address>[A-F0-9]+):(?<local_port>[A-F0-9]{4})
        \s*
        (?<remote_address>[A-F0-9]+):(?<remote_port>[A-F0-9]{4})
        \s*
        (?<state>[A-F0-9]{2})
        \s*
        [A-F0-9:]+
        \s*
        [A-F0-9:]+
        \s*
        [A-F0-9]+
        \s*
        [A-F0-9]+
        \s*
        [A-F0-9]+
        \s*
        (?<inode_number>\d+)
        ",
    )
    .unwrap();
    let mut addrs = Vec::new();
    while let n = reader.read_line(&mut line)?
        && n > 0
    {
        let Some(caps) = table_line.captures(&line) else {
            line.clear();
            continue;
        };

        if inodes.contains(&caps["inode_number"]) {
            let addr_str = hex_addr_to_ipv4_string(&caps["local_address"], &caps["local_port"]);
            addrs.push(SocketAddr::from_str(&addr_str).expect("Invalid Address"));
        };
        line.clear();
    }
    Ok(addrs)
}

// the address is hex in BE format
fn hex_addr_to_ipv4_string(address: &str, port: &str) -> String {
    assert_eq!(address.len(), 8);
    assert_eq!(port.len(), 4);

    let mut s = String::new();

    for i in (0..address.len()).step_by(2).rev() {
        let hex = &address[i..i + 2];
        s.push_str(
            &u8::from_str_radix(hex, 16)
                .expect("Should be valid hex")
                .to_string(),
        );
        if i > 0 {
            s.push('.');
        }
    }

    s.push(':');
    s.push_str(
        &u32::from_str_radix(port, 16)
            .expect("Port is invalid")
            .to_string(),
    );

    s
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hex_to_ip_string() {
        let hex_addr = "0100007F"; // 127.0.0.1
        let hex_port = "1F90"; // 8080
        assert_eq!(
            hex_addr_to_ipv4_string(hex_addr, hex_port),
            "127.0.0.1:8080"
        );
    }
}
