use std::io;
use std::process;

#[derive(Debug, PartialEq)]
pub struct ExecutorError {
    msg: String,
}

impl ExecutorError {
    fn new<S: Into<String>>(msg: S) -> ExecutorError {
        ExecutorError { msg: msg.into() }
    }
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for ExecutorError {}

pub trait Executor: Send {
    fn execute(&self, command: &str) -> Result<String, ExecutorError>;
}

struct ExecutorImpl<F>
where
    F: Fn(&str, &[String]) -> io::Result<process::Output> + Send,
{
    execute_fn: F,
}

impl<F> Executor for ExecutorImpl<F>
where
    F: Fn(&str, &[String]) -> io::Result<process::Output> + Send,
{
    fn execute(&self, command: &str) -> Result<String, ExecutorError> {
        let parts = match shlex::split(command) {
            Some(parts) => parts,
            None => return Err(ExecutorError::new("Invalid command string")),
        };

        if parts.is_empty() {
            return Err(ExecutorError::new("No command specified"));
        }

        match (self.execute_fn)(&parts[0], &parts[1..]) {
            Ok(output) => match output.status.success() {
                true => Ok(String::from_utf8(output.stdout).unwrap()),
                false => Err(ExecutorError::new("Failure exit code")),
            },
            Err(e) => Err(ExecutorError::new(format!("IO Error: {}", e))),
        }
    }
}

#[allow(dead_code)]
pub fn build() -> Box<dyn Executor> {
    Box::new(ExecutorImpl {
        execute_fn: |cmd: &str, args: &[String]| process::Command::new(cmd).args(args).output(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    #[cfg(target_os = "linux")]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(target_os = "windows")]
    use std::os::windows::process::ExitStatusExt;

    #[test]
    fn test_input_invalid() {
        let executor = ExecutorImpl {
            execute_fn: |_, _| Err(io::Error::from(io::ErrorKind::Unsupported)),
        };

        assert_eq!(
            ExecutorError::new("Invalid command string"),
            executor.execute("\"").unwrap_err()
        );

        assert_eq!(
            ExecutorError::new("No command specified"),
            executor.execute("").unwrap_err()
        );
    }

    #[test]
    fn test_io_error() {
        let executor = ExecutorImpl {
            execute_fn: |_, _| Err(io::Error::from(io::ErrorKind::Unsupported)),
        };

        assert_eq!(
            ExecutorError::new("IO Error: unsupported"),
            executor.execute("cmd").unwrap_err()
        );
    }

    #[test]
    fn test_failed_exit_code() {
        let executor = ExecutorImpl {
            execute_fn: |cmd, args| {
                assert_eq!("command", cmd);
                assert_eq!(Vec::<String>::new(), args);
                Ok(process::Output {
                    status: process::ExitStatus::from_raw(1),
                    stdout: vec![],
                    stderr: vec![],
                })
            },
        };

        assert_eq!(
            ExecutorError::new("Failure exit code"),
            executor.execute("command").unwrap_err()
        );
    }

    #[test]
    fn test_success() {
        let executor = ExecutorImpl {
            execute_fn: |cmd, args| {
                assert_eq!("command", cmd);
                assert_eq!(vec!["arg1", "arg2"], args);
                Ok(process::Output {
                    status: process::ExitStatus::from_raw(0),
                    stdout: "output".as_bytes().to_owned(),
                    stderr: vec![],
                })
            },
        };

        assert_eq!("output", executor.execute("command arg1 arg2").unwrap());
    }
}
