use ethers::core::types::H256;
use prometheus::IntCounter;
use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use nomad_base::{CachingHome, IncrementalMerkleSync, NomadDB, UpdaterError};
use nomad_core::{Home, SignedUpdate, Signers, Update};
use tokio::{task::JoinHandle, time::sleep};
use tracing::{error, info, info_span, instrument::Instrumented, Instrument};

#[derive(Debug)]
pub(crate) struct UpdateProducer {
    home: Arc<CachingHome>,
    merkle_sync: Arc<IncrementalMerkleSync>,
    db: NomadDB,
    signer: Arc<Signers>,
    interval_seconds: u64,
    signed_attestation_count: IntCounter,
}

impl UpdateProducer {
    pub(crate) fn new(
        home: Arc<CachingHome>,
        db: NomadDB,
        signer: Arc<Signers>,
        interval_seconds: u64,
        signed_attestation_count: IntCounter,
    ) -> Self {
        let merkle_sync = Arc::new(IncrementalMerkleSync::from_disk(db.clone()));

        Self {
            home,
            merkle_sync,
            db,
            signer,
            interval_seconds,
            signed_attestation_count,
        }
    }

    /// Return latest committed root (new root of last confirmed update)
    fn latest_committed_root(&self) -> Result<H256> {
        // If db latest root is empty, this will produce `H256::default()`
        // which is equal to `H256::zero()`
        Ok(self.db.retrieve_latest_root()?.unwrap_or_default())
    }

    /// Store a pending update in the DB for potential submission.
    ///
    /// This does not produce update meta or update the latest update db value.
    /// It is used by update production and submission.
    fn store_produced_update(&self, update: &SignedUpdate) -> Result<()> {
        let existing_opt = self
            .db
            .retrieve_produced_update(update.update.previous_root)?;

        if let Some(existing) = existing_opt {
            if existing.update.new_root != update.update.new_root {
                error!("Updater attempted to store conflicting update. Existing update: {:?}. New conflicting update: {:?}.", &existing, &update);

                return Err(UpdaterError::ProducerConflictError {
                    existing: existing.update,
                    conflicting: update.update,
                }
                .into());
            }
        } else {
            self.db
                .store_produced_update(update.update.previous_root, update)?;
        }

        Ok(())
    }

    /// Spawn the updater's produce task.
    ///
    /// Note that all data retrieved from either contract calls or the
    /// updater's db are confirmed state in the chain, as both indexed data and
    /// contract state are retrieved with a timelag.
    pub(crate) fn spawn(self) -> Instrumented<JoinHandle<Result<()>>> {
        let span = info_span!("UpdateProducer");
        tokio::spawn(async move {
            loop {
                // We sleep at the top to make continues work fine
                sleep(Duration::from_secs(self.interval_seconds)).await;

                // Get home indexer's latest seen update from home. This call 
                // will only return a root from an update that is confirmed in 
                // the chain, as the updater indexer's timelag will ensure this.
                let last_committed = self.latest_committed_root()?;

                // The produced update is also confirmed state in the chain, as 
                // home indexing timelag for dispatched messages ensures this.
                let new_root = self.merkle_sync.tree.root();

                // If last committed root is same as current merkle root,
                // no update to produce
                if last_committed == new_root {
                    info!("No updates to sign. Waiting for new root building off of current root {:?}.", last_committed);
                    continue;
                }

                // Ensure we have not already signed a conflicting update.
                // Ignore suggested if we have.
                if let Some(existing) = self.db.retrieve_produced_update(last_committed)? {
                    if existing.update.new_root != new_root {
                        info!("Updater ignoring conflicting suggested update. Indicates chain awaiting already produced update. Existing update: {:?}. Suggested conflicting update: {} --> {}.", &existing, &last_committed, &new_root);
                    }

                    continue;
                }

                // If the suggested matches our local view, sign an update
                // and store it as locally produced
                let update = Update {
                    home_domain: self.home.local_domain(),
                    previous_root: last_committed,
                    new_root,
                };
                let signed = update.sign_with(self.signer.as_ref()).await?;

                self.signed_attestation_count.inc();

                let hex_signature = format!("0x{}", hex::encode(signed.signature.to_vec()));
                info!(
                    previous_root = ?signed.update.previous_root,
                    new_root = ?signed.update.new_root,
                    hex_signature = %hex_signature,
                    "Storing new update in DB for broadcast"
                );

                // Once we have stored signed update in db, updater can 
                // never produce a double update building off the same 
                // previous root (we check db each time we produce new 
                // signed update)
                self.store_produced_update(&signed)?
            }
        })
        .instrument(span)
    }
}
