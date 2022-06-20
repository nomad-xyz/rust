use color_eyre::Result;

/// Transaction poller for submitting PersistentTransaction
#[derive(Debug, Clone)]
pub struct TxSender;

impl TxSender {
    pub fn new() -> Self {
        Self {}
    }

    /// TODO:
    pub async fn run(&self) -> Result<()> {
        unimplemented!()
    }
}

impl std::fmt::Display for TxSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
