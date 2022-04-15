use log::{Level, Log, Metadata, Record};

const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

struct ConsoleLogger {}

impl Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with(CRATE_NAME)
    }

    fn log(&self, record: &Record) {
        let metadata = record.metadata();
        if self.enabled(metadata) {
            let msg = format!(
                "[{}] [{}] [{}] {}",
                chrono::Local::now().to_rfc2822(),
                std::thread::current().name().expect("Thread has no name"),
                record.level(),
                record.args()
            );
            match metadata.level() {
                Level::Error | Level::Warn => eprintln!("{}", msg),
                _ => println!("{}", msg),
            }
        }
    }

    fn flush(&self) {}
}

pub fn init_logging() {
    log::set_boxed_logger(Box::new(ConsoleLogger {})).expect("Unable to initialize logger");
    log::set_max_level(log::LevelFilter::max());
}
