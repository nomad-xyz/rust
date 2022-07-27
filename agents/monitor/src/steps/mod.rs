use ethers::prelude::rand::{prelude::ThreadRng, Rng};

pub(crate) mod between;
pub(crate) mod combine;
pub(crate) mod dispatch_wait;
pub(crate) mod e2e;
pub(crate) mod producer;
pub(crate) mod relay_wait;
pub(crate) mod terminal;
pub(crate) mod update_wait;

#[derive(Debug)]
pub(crate) enum TaskResult<T> {
    Recoverable {
        task: T,
        err: eyre::Report,
    },
    Unrecoverable {
        err: eyre::Report,
        worth_logging: bool,
    },
}

// adds up to a second of random delay to cause production tasks to not be synced
pub(crate) fn noisy_sleep(approx_millis: u64) -> tokio::time::Sleep {
    let noise = ThreadRng::default().gen_range(0..1000u64);
    let duration = std::time::Duration::from_millis(approx_millis + noise);
    tokio::time::sleep(duration)
}
