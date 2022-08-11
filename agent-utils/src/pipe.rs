use std::fmt::Debug;

use eyre::bail;

use crate::aliases::*;

#[derive(Debug)]
pub struct Pipe<T> {
    rx: Faucet<T>,
    tx: Sink<T>,
    contents: Option<T>,
}

impl<T> Pipe<T>
where
    T: Debug + Send + Sync + 'static,
{
    pub fn new(rx: Faucet<T>, tx: Sink<T>, contents: Option<T>) -> Self {
        Self { rx, tx, contents }
    }

    pub fn read(&self) -> Option<&T> {
        self.contents.as_ref()
    }

    pub fn finish(&mut self) -> eyre::Result<()> {
        if let Some(contents) = self.contents.take() {
            self.tx.send(contents)?;
        }
        Ok(())
    }

    pub async fn next(&mut self) -> eyre::Result<&T> {
        self.finish()?;

        self.contents = self.rx.recv().await;
        if self.contents.is_none() {
            bail!("rx broke")
        }
        Ok(self.read().expect("checked"))
    }
}
