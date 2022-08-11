pub mod aliases;
pub mod init;
pub mod macros;
pub mod pipe;
pub mod utils;

use std::panic;

pub use aliases::*;

use tokio::task::JoinHandle;

#[derive(Debug)]
pub enum TaskResult<T> {
    Recoverable {
        task: T,
        err: eyre::Report,
    },
    Unrecoverable {
        err: eyre::Report,
        worth_logging: bool,
    },
}

pub trait ProcessStep: std::fmt::Display {
    fn spawn(self) -> Restartable<Self>
    where
        Self: 'static + Send + Sync + Sized;

    /// Run the task until it panics. Errors result in a task restart with the
    /// same channels. This means that an error causes the task to lose only
    /// the data that is in-scope when it faults.
    fn run_until_panic(self) -> JoinHandle<()>
    where
        Self: 'static + Send + Sync + Sized,
    {
        let task_description = format!("{}", self);
        tokio::spawn(async move {
            let mut handle = self.spawn();
            loop {
                let result = handle.await;

                let again = match result {
                    Ok(TaskResult::Recoverable { task, err }) => {
                        tracing::warn!(
                            error = %err,
                            task = task_description.as_str(),
                            "Restarting task",
                        );
                        task
                    }

                    Ok(TaskResult::Unrecoverable { err, worth_logging }) => {
                        if worth_logging {
                            tracing::error!(err = %err, task = task_description.as_str(), "Unrecoverable error encountered");
                        } else {
                            tracing::trace!(err = %err, task = task_description.as_str(), "Unrecoverable error encountered");
                        }
                        break;
                    }

                    Err(e) => {
                        let panic_res = e.try_into_panic();

                        if panic_res.is_err() {
                            tracing::trace!(
                                task = task_description.as_str(),
                                "Internal task cancelled",
                            );
                            break;
                        }
                        let p = panic_res.unwrap();
                        tracing::error!(task = task_description.as_str(), "Internal task panicked");
                        panic::resume_unwind(p);
                    }
                };

                utils::noisy_sleep(15_000).await;
                handle = again.spawn();
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{ProcessStep, TaskResult};

    struct RecoverableTask;
    impl std::fmt::Display for RecoverableTask {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "RecoverableTask")
        }
    }

    impl ProcessStep for RecoverableTask {
        fn spawn(self) -> crate::Restartable<Self>
        where
            Self: 'static + Send + Sync + Sized,
        {
            tokio::spawn(async move {
                TaskResult::Recoverable {
                    task: self,
                    err: eyre::eyre!("This error was recoverable"),
                }
            })
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_recovery() {
        let handle = RecoverableTask.run_until_panic();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        handle.abort();
        let result = handle.await;

        assert!(logs_contain("RecoverableTask"));
        assert!(logs_contain("Restarting task"));
        assert!(logs_contain("This error was recoverable"));
        assert!(result.is_err() && result.unwrap_err().is_cancelled());
    }

    struct UnrecoverableTask;
    impl std::fmt::Display for UnrecoverableTask {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "UnrecoverableTask")
        }
    }

    impl ProcessStep for UnrecoverableTask {
        fn spawn(self) -> crate::Restartable<Self>
        where
            Self: 'static + Send + Sync + Sized,
        {
            tokio::spawn(async move {
                TaskResult::Unrecoverable {
                    err: eyre::eyre!("This error was unrecoverable"),
                    worth_logging: true,
                }
            })
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_unrecoverable() {
        let handle = UnrecoverableTask.run_until_panic();
        let result = handle.await;
        assert!(logs_contain("UnrecoverableTask"));
        assert!(logs_contain("Unrecoverable error encountered"));
        assert!(logs_contain("This error was unrecoverable"));
        assert!(result.is_ok());
    }

    struct PanicTask;
    impl std::fmt::Display for PanicTask {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "PanicTask")
        }
    }

    impl ProcessStep for PanicTask {
        fn spawn(self) -> crate::Restartable<Self>
        where
            Self: 'static + Send + Sync + Sized,
        {
            tokio::spawn(async move { panic!("intentional panic :)") })
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_panic() {
        let handle = PanicTask.run_until_panic();
        let result = handle.await;
        assert!(logs_contain("PanicTask"));
        assert!(logs_contain("Internal task panicked"));
        assert!(result.is_err() && result.unwrap_err().is_panic());
    }
}
