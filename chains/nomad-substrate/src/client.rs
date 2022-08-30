use crate::configs::avail::avail::home;
use color_eyre::Result;
use ethers_core::types::Signature;
use nomad_core::{RawCommittedMessage, SignedUpdate, SignedUpdateWithMeta, Update, UpdateMeta};
use subxt::{Config, OnlineClient};

/// Nomad wrapper around `subxt::OnlineClient`
#[derive(Clone)]
pub struct NomadOnlineClient<T: Config>(OnlineClient<T>);

impl<T: Config> std::ops::Deref for NomadOnlineClient<T> {
    type Target = OnlineClient<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Config> From<OnlineClient<T>> for NomadOnlineClient<T> {
    fn from(client: OnlineClient<T>) -> Self {
        Self(client)
    }
}

impl<T: Config> NomadOnlineClient<T> {
    /// Fetch ordered signed updates from the specific `block_number`
    pub async fn fetch_sorted_updates_for_block(
        &self,
        block_number: u32,
    ) -> Result<Vec<SignedUpdateWithMeta>> {
        // Get hash for block number
        let hash = self
            .rpc()
            .block_hash(Some(block_number.into()))
            .await?
            .unwrap();

        // Get updates from block
        let update_events_res: Result<Vec<_>, _> = self
            .events()
            .at(Some(hash))
            .await?
            .find::<home::events::Update>() // TODO: remove dependency on avail metadata
            .into_iter()
            .collect();

        let update_events = update_events_res?;

        // TODO: sort events

        // Map update events into SignedUpdates with meta
        Ok(update_events
            .into_iter()
            .map(|ev| {
                let signature = Signature::try_from(ev.signature.as_ref())
                    .expect("chain accepted invalid signature");

                SignedUpdateWithMeta {
                    signed_update: SignedUpdate {
                        update: Update {
                            home_domain: ev.home_domain,
                            previous_root: ev.previous_root,
                            new_root: ev.new_root,
                        },
                        signature,
                    },
                    metadata: UpdateMeta {
                        block_number: block_number as u64,
                        timestamp: None,
                    },
                }
            })
            .collect())
    }

    /// Fetch ordered signed updates from the specific `block_number`
    pub async fn fetch_sorted_messages_for_block(
        &self,
        block_number: u32,
    ) -> Result<Vec<RawCommittedMessage>> {
        // Get hash for block number
        let hash = self
            .rpc()
            .block_hash(Some(block_number.into()))
            .await?
            .unwrap();

        // Get dispatch events from block
        let dispatch_events_res: Result<Vec<_>, _> = self
            .events()
            .at(Some(hash))
            .await?
            .find::<home::events::Dispatch>() // TODO: remove dependency on avail metadata
            .into_iter()
            .collect();

        let dispatch_events = dispatch_events_res?;

        // TODO: sort events

        // Map dispatches into raw committed messages
        Ok(dispatch_events
            .into_iter()
            .map(|ev| RawCommittedMessage {
                leaf_index: ev.leaf_index,
                committed_root: ev.committed_root,
                message: ev.message,
            })
            .collect())
    }
}