#![warn(clippy::all)]

use structopt::StructOpt;

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
    let config = regite::config::load_config(&args.config).unwrap();

    let mut regite = regite::Regite::new(config);
    regite.start();
    regite.join();
}
