use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct RunningState {
    join_handle: Option<JoinHandle<()>>,
    exit_notifier: Sender<()>,
}

enum RunnerState {
    Unknown,
    Pending(Duration, Box<dyn Fn() + Send + 'static>),
    Running(RunningState),
    Stopped,
}

pub struct Runner {
    state: RunnerState,
}

impl Runner {
    pub fn new(interval: Duration, run_fn: Box<dyn Fn() + Send + 'static>) -> Runner {
        Runner {
            state: RunnerState::Pending(interval, run_fn),
        }
    }

    pub fn start(&mut self) {
        let (interval, run_fn) = match std::mem::replace(&mut self.state, RunnerState::Unknown) {
            RunnerState::Pending(interval, run_fn) => (interval, run_fn),
            _ => panic!("A runner can only be started once"),
        };

        let (tx, rx) = channel();
        let join_handle = thread::spawn(move || loop {
            let start_time = Instant::now();
            run_fn();

            let mut elapsed = start_time.elapsed();
            while elapsed > interval {
                elapsed -= interval;
            }
            if rx.recv_timeout(interval - elapsed).is_ok() {
                break;
            }
        });

        self.state = RunnerState::Running(RunningState {
            join_handle: Some(join_handle),
            exit_notifier: tx,
        });
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        let exit_notifier = match &self.state {
            RunnerState::Running(running_state) => &running_state.exit_notifier,
            _ => panic!("A runner can only be stopped while running"),
        };

        exit_notifier.send(()).expect("Couldn't notify thread");
    }

    pub fn join(&mut self) {
        let join_handle = match &mut self.state {
            RunnerState::Running(running_state) => std::mem::take(&mut running_state.join_handle)
                .expect("join_handle shouldn't be none"),
            _ => panic!("A runner can only be joined while running"),
        };
        join_handle.join().expect("Couldn't join thread");
        self.state = RunnerState::Stopped;
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
        let mut runner = Runner::new(
            Duration::from_millis(1),
            Box::new(move || {
                run_count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

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
        let mut runner = Runner::new(
            Duration::from_millis(2),
            Box::new(move || {
                run_count_clone.fetch_add(1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(3));
            }),
        );

        runner.start();
        thread::sleep(Duration::from_millis(5));
        runner.stop();
        runner.join();

        assert_eq!(2, run_count.load(Ordering::SeqCst));
    }

    #[test]
    #[should_panic(expected = "A runner can only be started once")]
    fn test_cannot_start_twice() {
        let mut runner = Runner::new(Duration::from_millis(1), Box::new(|| {}));

        runner.start();
        runner.start();
    }

    #[test]
    #[should_panic(expected = "A runner can only be stopped while running")]
    fn test_cannot_stop_before_start() {
        let runner = Runner::new(Duration::from_millis(1), Box::new(|| {}));

        runner.stop();
    }

    #[test]
    #[should_panic(expected = "A runner can only be joined while running")]
    fn test_cannot_join_before_start() {
        let mut runner = Runner::new(Duration::from_millis(1), Box::new(|| {}));

        runner.join();
    }

    #[test]
    #[should_panic(expected = "A runner can only be joined while running")]
    fn test_cannot_join_twice() {
        let mut runner = Runner::new(Duration::from_millis(1), Box::new(|| {}));

        runner.start();
        runner.stop();
        runner.join();
        runner.join();
    }

    #[test]
    fn test_state_should_not_be_unknown() {
        let mut runner = Runner::new(Duration::from_millis(1), Box::new(|| {}));

        assert!(!matches!(runner.state, RunnerState::Unknown));
        runner.start();
        assert!(!matches!(runner.state, RunnerState::Unknown));
        runner.stop();
        assert!(!matches!(runner.state, RunnerState::Unknown));
        runner.join();
        assert!(!matches!(runner.state, RunnerState::Unknown));
    }
}
