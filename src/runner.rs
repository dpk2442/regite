#![allow(dead_code)]

use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct Runner<F>
where
    F: Fn() + Send + 'static,
{
    interval: Duration,
    run_fn: Option<F>,
    join_handle: Option<JoinHandle<()>>,
    exit_notifier: Option<Sender<()>>,
}

impl<F> Runner<F>
where
    F: Fn() + Send + 'static,
{
    pub fn new(interval: Duration, run_fn: F) -> Runner<F> {
        Runner {
            interval,
            run_fn: Some(run_fn),
            join_handle: None,
            exit_notifier: None,
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = channel();
        let interval = self.interval;
        let run_fn = std::mem::take(&mut self.run_fn).expect("Cannot start the runner twice");
        self.exit_notifier = Some(tx);
        self.join_handle = Some(thread::spawn(move || loop {
            let start_time = Instant::now();
            run_fn();

            let mut elapsed = start_time.elapsed();
            while elapsed > interval {
                elapsed -= interval;
            }
            if rx.recv_timeout(interval - elapsed).is_ok() {
                break;
            }
        }));
    }

    pub fn stop(&mut self) {
        if let Some(exit_notifier) = &self.exit_notifier {
            exit_notifier.send(()).expect("Couldn't notify thread");
            self.exit_notifier = None;
        }
    }

    pub fn join(&mut self) {
        if let Some(join_handle) = std::mem::take(&mut self.join_handle) {
            join_handle.join().expect("Couldn't join thread");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    #[test]
    fn test_fast_function() {
        let run_count = Arc::new(AtomicU32::new(0));
        let run_count_clone = run_count.clone();
        let mut runner = Runner::new(Duration::from_millis(1), move || {
            run_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        runner.start();
        thread::sleep(Duration::from_millis(5));
        runner.stop();
        runner.join();

        assert_eq!(5, run_count.load(Ordering::SeqCst));
    }

    #[test]
    fn test_overrun_function() {
        let run_count = Arc::new(AtomicU32::new(0));
        let run_count_clone = run_count.clone();
        let mut runner = Runner::new(Duration::from_millis(2), move || {
            run_count_clone.fetch_add(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(3));
        });

        runner.start();
        thread::sleep(Duration::from_millis(5));
        runner.stop();
        runner.join();

        assert_eq!(2, run_count.load(Ordering::SeqCst));
    }
}
