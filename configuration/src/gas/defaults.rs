use crate::{
    BridgeGasConfig, BridgeRouterGasLimits, ConnectionManagerGasLimits, CoreGasConfig,
    EthHelperGasLimits, HomeGasLimits, HomeUpdateGasLimit, NomadGasConfig, ReplicaGasLimits,
};

pub const EVM_DEFAULT: NomadGasConfig = NomadGasConfig {
    core: CoreGasConfig {
        home: HomeGasLimits {
            update: HomeUpdateGasLimit {
                per_message: 2_000,
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
};
