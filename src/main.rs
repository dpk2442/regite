#![warn(clippy::all)]

use std::sync::{Arc, Condvar, Mutex};

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
    regite::logging::init_logging();

    let args = Args::from_args();
    log::info!("Loading config from {}", args.config);
    let config = regite::config::load_config(&args.config).unwrap();

    log::info!("Starting background threads...");
    let mut regite = regite::Regite::new(config);
    regite.start();
    log::info!("Background threads started");

    #[allow(clippy::mutex_atomic)]
    let condvar_pair = Arc::new((Mutex::new(true), Condvar::new()));
    let condvar_pair_clone = condvar_pair.clone();
    ctrlc::set_handler(move || {
        let (lock, cvar) = &*condvar_pair_clone;
        let mut pending = lock.lock().unwrap();
        *pending = false;
        cvar.notify_one();
    })
    .expect("Error setting ctrl-c handler");

    let (lock, cvar) = &*condvar_pair;
    let _guard = cvar
        .wait_while(lock.lock().unwrap(), |pending| *pending)
        .unwrap();

    log::info!("Stopping background threads...");
    regite.stop();
    regite.join();
    log::info!("Background threads stopped");
}
