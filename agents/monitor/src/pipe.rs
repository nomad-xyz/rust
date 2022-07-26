use std::fmt::Debug;

use eyre::bail;
use nomad_ethereum::bindings::{
    home::{DispatchFilter, UpdateFilter},
    replica::{ProcessFilter, UpdateFilter as RelayFilter},
};

use crate::{annotate::WithMeta, Faucet, Sink};

#[derive(Debug)]
pub(crate) struct Pipe<T> {
    rx: Faucet<T>,
    tx: Sink<T>,
    contents: Option<T>,
}

pub(crate) type DispatchPipe = Pipe<WithMeta<DispatchFilter>>;
pub(crate) type UpdatePipe = Pipe<WithMeta<UpdateFilter>>;
pub(crate) type RelayPipe = Pipe<WithMeta<RelayFilter>>;
pub(crate) type ProcessPipe = Pipe<WithMeta<ProcessFilter>>;

impl<T> Pipe<T>
where
    T: Debug + Send + Sync + 'static,
{
    pub(crate) fn new(rx: Faucet<T>, tx: Sink<T>, contents: Option<T>) -> Self {
        Self { rx, tx, contents }
    }

    pub(crate) fn read(&self) -> Option<&T> {
        self.contents.as_ref()
    }

    pub(crate) fn finish(&mut self) -> eyre::Result<()> {
        if let Some(contents) = self.contents.take() {
            self.tx.send(contents)?;
        }
        Ok(())
    }

    pub(crate) async fn next(&mut self) -> eyre::Result<Option<&T>> {
        self.finish()?;

        self.contents = self.rx.recv().await;
        if self.contents.is_none() {
            bail!("Rx Broke")
        }
        Ok(self.read())
    }
}
