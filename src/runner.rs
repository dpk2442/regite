use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

struct RunningState {
    join_handle: Option<JoinHandle<()>>,
    exit_notifier: Sender<()>,
}

enum RunnerState {
    Pending(String, Duration, Box<dyn Fn() + Send + 'static>),
    Running(RunningState),
    Stopped,
}

pub struct Runner {
    state: Option<RunnerState>,
}

impl Runner {
    pub fn new(name: String, interval: Duration, run_fn: Box<dyn Fn() + Send + 'static>) -> Runner {
        Runner {
            state: Some(RunnerState::Pending(name, interval, run_fn)),
        }
    }

    pub fn start(&mut self) {
        let (name, interval, run_fn) = match std::mem::take(&mut self.state) {
            Some(RunnerState::Pending(name, interval, run_fn)) => (name, interval, run_fn),
            _ => panic!("A runner can only be started once"),
        };

        let (tx, rx) = channel();
        let join_handle = thread::Builder::new()
            .name(name)
            .spawn(move || loop {
                let start_time = Instant::now();
                run_fn();

                let mut elapsed = start_time.elapsed();
                while elapsed > interval {
                    elapsed -= interval;
                }
                if rx.recv_timeout(interval - elapsed).is_ok() {
                    break;
                }
            })
            .expect("Couldn't spawn thread");

        self.state = Some(RunnerState::Running(RunningState {
            join_handle: Some(join_handle),
            exit_notifier: tx,
        }));
    }

    pub fn stop(&self) {
        let exit_notifier = match &self.state {
            Some(RunnerState::Running(running_state)) => &running_state.exit_notifier,
            _ => panic!("A runner can only be stopped while running"),
        };

        exit_notifier.send(()).expect("Couldn't notify thread");
    }

    pub fn join(&mut self) {
        let join_handle = match &mut self.state {
            Some(RunnerState::Running(running_state)) => {
                std::mem::take(&mut running_state.join_handle)
                    .expect("join_handle shouldn't be none")
            }
            _ => panic!("A runner can only be joined while running"),
        };
        join_handle.join().expect("Couldn't join thread");
        self.state = Some(RunnerState::Stopped);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic(expected = "A runner can only be started once")]
    fn test_cannot_start_twice() {
        let mut runner = Runner::new(
            "name".to_string(),
            Duration::from_millis(1),
            Box::new(|| {}),
        );

        runner.start();
        runner.start();
    }

    #[test]
    #[should_panic(expected = "A runner can only be stopped while running")]
    fn test_cannot_stop_before_start() {
        let runner = Runner::new(
            "name".to_string(),
            Duration::from_millis(1),
            Box::new(|| {}),
        );

        runner.stop();
    }

    #[test]
    #[should_panic(expected = "A runner can only be joined while running")]
    fn test_cannot_join_before_start() {
        let mut runner = Runner::new(
            "name".to_string(),
            Duration::from_millis(1),
            Box::new(|| {}),
        );

        runner.join();
    }

    #[test]
    #[should_panic(expected = "A runner can only be joined while running")]
    fn test_cannot_join_twice() {
        let mut runner = Runner::new(
            "name".to_string(),
            Duration::from_millis(1),
            Box::new(|| {}),
        );

        runner.start();
        runner.stop();
        runner.join();
        runner.join();
    }

    #[test]
    fn test_state_should_not_be_none() {
        let mut runner = Runner::new(
            "name".to_string(),
            Duration::from_millis(1),
            Box::new(|| {}),
        );

        assert!(runner.state.is_some());
        runner.start();
        assert!(runner.state.is_some());
        runner.stop();
        assert!(runner.state.is_some());
        runner.join();
        assert!(runner.state.is_some());
    }
}
