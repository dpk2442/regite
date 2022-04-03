use std::io;
use std::net;

use crate::config;

#[derive(Debug, PartialEq)]
pub struct MetricReporterError {
    msg: String,
}

impl MetricReporterError {
    fn new<S: Into<String>>(msg: S) -> MetricReporterError {
        MetricReporterError { msg: msg.into() }
    }
}

impl std::fmt::Display for MetricReporterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for MetricReporterError {}

pub trait MetricReporter: Send {
    fn report(&self, name: &str, value: &str, timestamp: u64) -> Result<(), MetricReporterError>;
}

struct MetricReporterImpl {
    prefix: String,
    hostname: String,
    send_fn: Box<dyn Fn(&str) -> io::Result<()> + Send>,
}

impl MetricReporter for MetricReporterImpl {
    fn report(&self, name: &str, value: &str, timestamp: u64) -> Result<(), MetricReporterError> {
        let metric = format!(
            "{}.{}.{} {} {}\n",
            self.prefix, self.hostname, name, value, timestamp
        );
        match (self.send_fn)(&metric) {
            Ok(()) => Ok(()),
            Err(e) => Err(MetricReporterError::new(format!("IO Error: {}", e))),
        }
    }
}

pub fn build(config: &config::General) -> Box<dyn MetricReporter> {
    let socket = net::UdpSocket::bind("[::]:0").expect("Unable to bind to ephemeral port");
    let address = config.graphite_address.clone();
    Box::new(MetricReporterImpl {
        prefix: config.prefix.to_string(),
        hostname: config.hostname.to_string(),
        send_fn: Box::new(move |s| socket.send_to(s.as_bytes(), &address).and(Ok(()))),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_formats() {
        let reporter = MetricReporterImpl {
            prefix: "prefix".to_string(),
            hostname: "hostname".to_string(),
            send_fn: Box::new(|s| {
                assert_eq!("prefix.hostname.name value 123\n", s);
                Ok(())
            }),
        };

        assert_eq!(Ok(()), reporter.report("name", "value", 123));
    }

    #[test]
    fn test_io_error() {
        let reporter = MetricReporterImpl {
            prefix: "prefix".to_string(),
            hostname: "hostname".to_string(),
            send_fn: Box::new(|_| Err(io::Error::from(io::ErrorKind::Unsupported))),
        };

        assert_eq!(
            MetricReporterError::new("IO Error: unsupported"),
            reporter.report("name", "value", 123).unwrap_err()
        );
    }
}
