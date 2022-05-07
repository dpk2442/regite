#![warn(clippy::all)]
#![cfg(target_os = "linux")]

use std::io::Read;
use std::net;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use regex::Regex;

use regite::{
    config::{Config, General, GraphiteConnectionType, Job, Output},
    Regite,
};

fn create_listener() -> (Arc<AtomicU32>, String) {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let socket = net::UdpSocket::bind("localhost:0").unwrap();
    let address = socket.local_addr().unwrap().to_string();
    thread::spawn(move || {
        let mut buf = [0; 100];
        loop {
            socket.recv(&mut buf).unwrap();
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    });
    (counter, address)
}

fn create_tcp_listener() -> (Arc<AtomicU32>, String) {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    let socket = net::TcpListener::bind("localhost:0").unwrap();
    let address = socket.local_addr().unwrap().to_string();
    thread::spawn(move || {
        let mut buf = [0; 100];
        loop {
            let (mut stream, _) = socket.accept().unwrap();
            let _ = stream.read(&mut buf).unwrap();
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    });
    (counter, address)
}

#[test]
fn test_date() {
    let socket = net::UdpSocket::bind("localhost:0").unwrap();
    let mut regite = Regite::new(Config {
        general: General {
            prefix: "prefix".to_string(),
            hostname: "host".to_string(),
            graphite_connection_type: GraphiteConnectionType::Udp,
            graphite_address: socket.local_addr().unwrap().to_string(),
        },
        job: vec![Job {
            name: "name".to_string(),
            interval: 1,
            command: "/usr/bin/env date +%s".to_string(),
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
    for _ in 0..5 {
        let len = socket.recv(&mut buf).unwrap();
        let msg = String::from_utf8_lossy(&buf[..len]);
        let captures = re.captures(&msg).unwrap();
        assert_eq!("prefix.host.date", &captures[1]);
        assert_eq!(captures[2], captures[3]);
    }

    regite.stop();
    regite.join();
}

#[test]
fn test_short_job() {
    let (counter, address) = create_listener();
    let mut regite = Regite::new(Config {
        general: General {
            prefix: "prefix".to_string(),
            hostname: "host".to_string(),
            graphite_connection_type: GraphiteConnectionType::Udp,
            graphite_address: address,
        },
        job: vec![Job {
            name: "name".to_string(),
            interval: 1,
            command: "/bin/bash -c \"echo 1\"".to_string(),
            regex: "(.+)".to_string(),
            output: vec![Output {
                name: "name".to_string(),
                value: "$1".to_string(),
            }],
        }],
    });

    regite.start();
    thread::sleep(Duration::from_secs(5));
    regite.stop();
    regite.join();

    assert_eq!(5, counter.load(Ordering::SeqCst));
}

#[test]
fn test_short_job_tcp() {
    let (counter, address) = create_tcp_listener();
    let mut regite = Regite::new(Config {
        general: General {
            prefix: "prefix".to_string(),
            hostname: "host".to_string(),
            graphite_connection_type: GraphiteConnectionType::Tcp,
            graphite_address: address,
        },
        job: vec![Job {
            name: "name".to_string(),
            interval: 1,
            command: "/bin/bash -c \"echo 1\"".to_string(),
            regex: "(.+)".to_string(),
            output: vec![Output {
                name: "name".to_string(),
                value: "$1".to_string(),
            }],
        }],
    });

    regite.start();
    thread::sleep(Duration::from_secs(5));
    regite.stop();
    regite.join();

    assert_eq!(5, counter.load(Ordering::SeqCst));
}

#[test]
fn test_long_job() {
    let (counter, address) = create_listener();
    let mut regite = Regite::new(Config {
        general: General {
            prefix: "prefix".to_string(),
            hostname: "host".to_string(),
            graphite_connection_type: GraphiteConnectionType::Udp,
            graphite_address: address,
        },
        job: vec![Job {
            name: "name".to_string(),
            interval: 1,
            command: "/bin/bash -c \"sleep 1.5; echo 1\"".to_string(),
            regex: "(.+)".to_string(),
            output: vec![Output {
                name: "name".to_string(),
                value: "$1".to_string(),
            }],
        }],
    });

    regite.start();
    thread::sleep(Duration::from_secs(5));

    // after 5 seconds, it should have run two times
    assert_eq!(2, counter.load(Ordering::SeqCst));

    regite.stop();
    regite.join();

    // ensure the final iteration has a chance to publish the UDP packet
    thread::sleep(Duration::from_secs(1));

    // there should have been a third run in progress
    assert_eq!(3, counter.load(Ordering::SeqCst));
}

#[test]
fn test_long_job_tcp() {
    let (counter, address) = create_tcp_listener();
    let mut regite = Regite::new(Config {
        general: General {
            prefix: "prefix".to_string(),
            hostname: "host".to_string(),
            graphite_connection_type: GraphiteConnectionType::Tcp,
            graphite_address: address,
        },
        job: vec![Job {
            name: "name".to_string(),
            interval: 1,
            command: "/bin/bash -c \"sleep 1.5; echo 1\"".to_string(),
            regex: "(.+)".to_string(),
            output: vec![Output {
                name: "name".to_string(),
                value: "$1".to_string(),
            }],
        }],
    });

    regite.start();
    thread::sleep(Duration::from_secs(5));

    // after 5 seconds, it should have run two times
    assert_eq!(2, counter.load(Ordering::SeqCst));

    regite.stop();
    regite.join();

    // ensure the final iteration has a chance to publish the TCP packet
    thread::sleep(Duration::from_secs(1));

    // there should have been a third run in progress
    assert_eq!(3, counter.load(Ordering::SeqCst));
}
