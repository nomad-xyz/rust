use crate::{
    BridgeGasConfig, BridgeRouterGasLimits, ConnectionManagerGasLimits, CoreGasConfig,
    EthHelperGasLimits, HomeGasLimits, HomeUpdateGasLimit, NomadGasConfig, ReplicaGasLimits,
};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Copy, serde::Serialize, PartialEq)]
pub struct EvmDefaultWrapper(pub NomadGasConfig);

impl<'de> Deserialize<'de> for EvmDefaultWrapper {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::default())
    }
}

impl Default for EvmDefaultWrapper {
    fn default() -> Self {
        Self(NomadGasConfig {
            core: CoreGasConfig {
                home: HomeGasLimits {
                    update: HomeUpdateGasLimit {
                        per_message: 10_000,
                        base: 100_000,
                    },
                    improper_update: HomeUpdateGasLimit {
                        per_message: 10_000,
                        base: 100_000,
                    },
                    double_update: 200_000,
                },
                replica: ReplicaGasLimits {
                    update: 140_000,
                    prove: 200_000,
                    process: 1_700_000,
                    prove_and_process: 1_900_000,
                    double_update: 200_000,
                },
                connection_manager: ConnectionManagerGasLimits {
                    owner_unenroll_replica: 120_000,
                    unenroll_replica: 120_000,
                },
            },
            bridge: BridgeGasConfig {
                bridge_router: BridgeRouterGasLimits { send: 500_000 },
                eth_helper: EthHelperGasLimits {
                    send: 800_000,
                    send_to_evm_like: 800_000,
                },
            },
        })
    }
}
