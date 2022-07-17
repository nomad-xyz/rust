use ethers::prelude::LogMeta;

pub(crate) struct Annotated<T> {
    pub(crate) log: T,
    pub(crate) meta: LogMeta,
}

impl<T> From<(T, LogMeta)> for Annotated<T> {
    fn from((log, meta): (T, LogMeta)) -> Self {
        Self { log, meta }
    }
}
