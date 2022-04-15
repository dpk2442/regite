#![warn(clippy::all)]
#![cfg(target_os = "linux")]

use std::net;

use regex::Regex;

use regite::{
    config::{Config, General, Job, Output},
    Regite,
};

#[test]
fn test_date() {
    let socket = net::UdpSocket::bind("localhost:0").unwrap();
    let mut regite = Regite::new(Config {
        general: General {
            prefix: "prefix".to_string(),
            hostname: "host".to_string(),
            graphite_address: socket.local_addr().unwrap().to_string(),
        },
        job: vec![Job {
            name: "name".to_string(),
            interval: 1,
            command: "/usr/bin/date +%s".to_string(),
            regex: "(.+)".to_string(),
            output: vec![Output {
                name: "date".to_string(),
                value: "$1".to_string(),
            }],
        }],
    });

    let re = Regex::new("([^ ]+) (\\d+) (\\d+)").unwrap();
    let mut buf = [0; 100];

    regite.start();
    for _ in 0..3 {
        let len = socket.recv(&mut buf).unwrap();
        let msg = String::from_utf8_lossy(&buf[..len]);
        let captures = re.captures(&msg).unwrap();
        assert_eq!("prefix.host.date", &captures[1]);
        assert_eq!(captures[2], captures[3]);
    }

    regite.stop();
    regite.join();
}
