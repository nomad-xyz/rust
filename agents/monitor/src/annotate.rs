use ethers::prelude::LogMeta;

#[derive(Debug)]
pub(crate) struct WithMeta<T>
where
    T: std::fmt::Debug,
{
    pub(crate) log: T,
    pub(crate) meta: LogMeta,
}

impl<T> From<(T, LogMeta)> for WithMeta<T>
where
    T: std::fmt::Debug,
{
    fn from((log, meta): (T, LogMeta)) -> Self {
        Self { log, meta }
    }
}
