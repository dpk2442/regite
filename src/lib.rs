#![warn(clippy::all)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod config;
mod executor;
mod metric;
mod parser;
mod runner;

pub struct Regite {
    runners: Vec<runner::Runner>,
}

impl Regite {
    pub fn new(config: config::Config) -> Regite {
        let prefix = format!("{}.{}", config.general.prefix, config.general.hostname);
        let mut runners = Vec::with_capacity(config.job.len());

        for job in &config.job {
            let command = job.command.clone();
            let executor = executor::build();
            let parser = parser::Parser::new(&prefix, &job.regex, &job.output);
            let metrics = metric::build(&config.general);
            runners.push(runner::Runner::new(
                Duration::from_secs(job.interval),
                Box::new(move || {
                    let start_time = SystemTime::now();
                    let output = match executor.execute(&command) {
                        Ok(output) => output,
                        Err(e) => {
                            eprintln!("Error: {:?}", e);
                            return;
                        }
                    };

                    let epoch_time = start_time
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs();
                    for (name, value) in parser.parse(&output) {
                        if let Err(e) = metrics.report(&name, &value, epoch_time) {
                            eprintln!("Error: {:?}", e);
                        }
                    }
                }),
            ));
        }

        Regite { runners }
    }

    pub fn start(&mut self) {
        for runner in &mut self.runners {
            runner.start();
        }
    }

    pub fn stop(&mut self) {
        for runner in &mut self.runners {
            runner.stop();
        }
    }

    pub fn join(&mut self) {
        for runner in &mut self.runners {
            runner.join();
        }
    }
}
