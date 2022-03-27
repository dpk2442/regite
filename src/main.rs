#![warn(clippy::all)]

use std::time::Duration;

use structopt::StructOpt;

mod config;
mod executor;
mod metric;
mod parser;
mod runner;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "regite",
    about = "Convert the output of scripts into graphite metrics"
)]
struct Args {
    #[structopt(
        short,
        long,
        value_name = "FILE",
        help = "Sets the config file to use",
        default_value = "/etc/regite.toml"
    )]
    pub config: String,
}

fn main() {
    let args = Args::from_args();
    let config = config::load_config(&args.config).unwrap();

    let prefix = format!("{}.{}", config.general.prefix, config.general.hostname);
    let mut runners = Vec::with_capacity(config.job.len());

    for job in &config.job {
        let command = job.command.clone();
        let executor = executor::build();
        let parser = parser::Parser::new(&prefix, &job.regex, &job.output);
        let metrics = metric::build(&config.general);
        runners.push(runner::Runner::new(
            Duration::from_secs(job.interval),
            move || {
                let output = match executor.execute(&command) {
                    Ok(output) => output,
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return;
                    }
                };

                for (name, value) in parser.parse(&output) {
                    if let Err(e) = metrics.report(&name, &value, 0) {
                        eprintln!("Error: {:?}", e);
                    }
                }
            },
        ));
    }

    for runner in &mut runners {
        runner.start();
    }

    for runner in &mut runners {
        runner.join();
    }
}
