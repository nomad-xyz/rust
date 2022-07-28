use annotate::WithMeta;
use futures_util::future::select_all;
use nomad_ethereum::bindings::{
    home::{DispatchFilter, UpdateFilter},
    replica::{ProcessFilter, UpdateFilter as RelayFilter},
};
use std::{collections::HashMap, panic, sync::Arc, time::Duration};
use steps::TaskResult;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tracing::info_span;

use ethers::prelude::{Http, Provider as EthersProvider};

pub(crate) mod annotate;
pub(crate) mod domain;
pub(crate) mod faucets;
pub(crate) mod init;
pub(crate) mod macros;
pub(crate) mod metrics;
pub(crate) mod pipe;
pub(crate) mod steps;
pub(crate) mod utils;

pub(crate) type Provider = ethers::prelude::TimeLag<EthersProvider<Http>>;
pub(crate) type ArcProvider = Arc<Provider>;
// pub(crate) type ProviderError = ContractError<Provider>;

pub(crate) type Restartable<Task> = JoinHandle<TaskResult<Task>>;

pub(crate) type Faucet<T> = UnboundedReceiver<T>;
pub(crate) type Sink<T> = UnboundedSender<T>;

pub(crate) type DispatchFaucet = Faucet<WithMeta<DispatchFilter>>;
pub(crate) type UpdateFaucet = Faucet<WithMeta<UpdateFilter>>;
pub(crate) type RelayFaucet = Faucet<WithMeta<RelayFilter>>;
pub(crate) type ProcessFaucet = Faucet<WithMeta<ProcessFilter>>;
pub(crate) type DispatchSink = Sink<WithMeta<DispatchFilter>>;
// pub(crate) type UpdateSink = Sink<WithMeta<UpdateFilter>>;
pub(crate) type RelaySink = Sink<WithMeta<RelayFilter>>;
pub(crate) type ProcessSink = Sink<WithMeta<ProcessFilter>>;

pub(crate) type NetworkMap<'a, T> = HashMap<&'a str, T>;
pub(crate) type HomeReplicaMap<'a, T> = HashMap<&'a str, HashMap<&'a str, T>>;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init::init_tracing();
    {
        let monitor = info_span!("MonitorBootup").in_scope(|| {
            let monitor = init::monitor()?;
            tracing::info!("setup complete!");
            Ok::<_, eyre::Report>(monitor)
        })?;

        let _http = monitor.run_http_server();

        let mut faucets = monitor.producers();

        monitor.run_betweens(&mut faucets);
        monitor.run_dispatch_to_update(&mut faucets);
        monitor.run_update_to_relay(&mut faucets);
        monitor.run_relay_to_process(&mut faucets);
        monitor.run_e2e(&mut faucets);

        // sink em
        let tasks = monitor.run_terminals(faucets);

        tracing::info!("tasks started");

        // run until there's a failure of a terminal
        // this would imply there is a series of upstream channel failures
        let (_, _, _) = select_all(tasks).await;
    }

    Ok(())
}

pub(crate) trait ProcessStep: std::fmt::Display {
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

                tokio::time::sleep(Duration::from_secs(15)).await;
                handle = again.spawn();
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{steps::TaskResult, ProcessStep};

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
