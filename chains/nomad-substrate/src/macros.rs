macro_rules! codegen_home {
    ($name:ident, $mod:ident, $config:ident) => {
        use subxt::{Config, OnlineClient, ext::sp_runtime::traits::Header};
        use nomad_core::{CommonIndexer, SignedUpdateWithMeta};
        use color_eyre::{eyre::eyre, Result};
        use futures::{stream::FuturesOrdered, StreamExt};
        use ethers_core::types::Signature;

        affix::paste! {
            pub struct [<$name Home>]<T: Config> {
                api: OnlineClient<T>,
                signer: std::sync::Arc<crate::SubstrateSigner<T>>,
                domain: u32,
                name: String,
            }

            impl<T: Config> std::ops::Deref for [<$name Home>]<T> {
                type Target = OnlineClient<T>;
                fn deref(&self) -> &Self::Target {
                    &self.api
                }
            }

            impl<T: Config> [<$name Home>]<T> {
                pub async fn new(
                    api: OnlineClient<T>,
                    signer: std::sync::Arc<crate::SubstrateSigner<T>>,
                    domain: u32,
                    name: &str,
                ) -> Result<Self> {
                    Ok(Self {
                        api,
                        signer,
                        domain,
                        name: name.to_owned(),
                    })
                }

                pub async fn base(&self) -> Result<[<$mod>]::runtime_types::nomad_base::NomadBase> {
                    let base_address = [<$mod>]::storage().home().base();
                    Ok(self.storage().fetch(&base_address, None).await?.unwrap())
                }
            }

            impl<T: Config> std::fmt::Debug for [<$name Home>]<T> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "SubstrateHome {{ domain: {}, name: {} }}",
                        self.domain, self.name,
                    )
                }
            }

            impl<T: Config> std::fmt::Display for [<$name Home>]<T> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(
                        f,
                        "SubstrateHome {{ domain: {}, name: {} }}",
                        self.domain, self.name,
                    )
                }
            }

            #[async_trait::async_trait]
            impl<T: Config + Send + Sync> CommonIndexer for [<$name Home>]<T>
            where
                T::BlockNumber: std::convert::TryInto<u32> + Send + Sync,
            {
                #[tracing::instrument(err, skip(self))]
                async fn get_block_number(&self) -> Result<u32> {
                    let header = self.rpc().header(None).await?.unwrap();
                    let u32_header = (*header.number()).try_into();

                    if let Ok(h) = u32_header {
                        Ok(h)
                    } else {
                        Err(eyre!("Failed to convert block number to u32"))
                    }
                }

                #[tracing::instrument(err, skip(self))]
                async fn fetch_sorted_updates(&self, from: u32, to: u32) -> Result<Vec<SignedUpdateWithMeta>> {
                    // Create future for fetching block hashes for range
                    let numbers_and_hash_futs: FuturesOrdered<_> = (from..to)
                        .map(|n| async move { (n, self.rpc().block_hash(Some(n.into())).await) })
                        .collect();

                    // Await and block hash requests
                    let numbers_and_hashes: Vec<_> = numbers_and_hash_futs
                        .collect::<Vec<_>>()
                        .await
                        .into_iter()
                        .map(|r| (r.0, r.1.unwrap().unwrap())) // TODO: is this safe to unwrap  (log RPC err)?
                        .collect();

                    // Get futures for events for each block's hash
                    let numbers_and_event_futs: FuturesOrdered<_> = numbers_and_hashes
                        .into_iter()
                        .map(|(n, h)| async move {
                            let events_api = self.events();
                            (n, events_api.at(Some(h)).await)
                        })
                        .collect();

                    // Await event requests and filter only update events
                    let numbers_and_update_events: Vec<_> = numbers_and_event_futs
                        .collect::<Vec<_>>()
                        .await
                        .into_iter()
                        .map(|(n, r)| {
                            let events_for_block = r.unwrap();
                            events_for_block
                                .find::<[<$mod>]::home::events::Update>()
                                .map(|r| (n, r.unwrap())) // TODO: is this safe to unwrap (log RPC err)?
                                .collect::<Vec<_>>()
                        })
                        .flatten()
                        .collect();

                    // Map update events into SignedUpdates with meta
                    Ok(numbers_and_update_events
                        .into_iter()
                        .map(|(n, e)| {
                            let signature = Signature::try_from(e.signature.as_ref())
                                .expect("chain accepted invalid signature");

                            nomad_core::SignedUpdateWithMeta {
                                signed_update: nomad_core::SignedUpdate {
                                    update: nomad_core::Update {
                                        home_domain: e.home_domain,
                                        previous_root: e.previous_root,
                                        new_root: e.new_root,
                                    },
                                    signature,
                                },
                                metadata: nomad_core::UpdateMeta {
                                    block_number: n as u64,
                                    timestamp: None,
                                },
                            }
                        })
                        .collect())
                }
            }
        }
    }
}
