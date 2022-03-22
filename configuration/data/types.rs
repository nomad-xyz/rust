#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "AddConfig")]
    pub type AppConfig;

    #[wasm_bindgen(typescript_type = "NomadLocator")]
    pub type NomadLocator;

    #[wasm_bindgen(typescript_type = "LogConfig")]
    pub type LogConfig;

    #[wasm_bindgen(typescript_type = "IndexConfig")]
    pub type IndexConfig;

    #[wasm_bindgen(typescript_type = "BaseAgentConfig")]
    pub type BaseAgentConfig;

    #[wasm_bindgen(typescript_type = "AgentConfig")]
    pub type AgentConfig;

    #[wasm_bindgen(typescript_type = "Proxy")]
    pub type Proxy;

    #[wasm_bindgen(typescript_type = "EvmCoreContracts")]
    pub type EvmCoreContracts;

    #[wasm_bindgen(typescript_type = "CoreContracts")]
    pub type CoreContracts;

    #[wasm_bindgen(typescript_type = "DeployedCustomToken")]
    pub type DeployedCustomToken;

    #[wasm_bindgen(typescript_type = "EvmBridgeContracts")]
    pub type EvmBridgeContracts;

    #[wasm_bindgen(typescript_type = "BridgeContracts")]
    pub type BridgeContracts;

    #[wasm_bindgen(typescript_type = "Governance")]
    pub type Governance;

    #[wasm_bindgen(typescript_type = "ContractConfig")]
    pub type ContractConfig;

    #[wasm_bindgen(typescript_type = "NetworkSpecs")]
    pub type NetworkSpecs;

    #[wasm_bindgen(typescript_type = "CustomTokenSpecifier")]
    pub type CustomTokenSpecifier;

    #[wasm_bindgen(typescript_type = "BridgeConfiguration")]
    pub type BridgeConfiguration;

    #[wasm_bindgen(typescript_type = "Domain")]
    pub type Domain;

    #[wasm_bindgen(typescript_type = "NetworkInfo")]
    pub type NetworkInfo;

    #[wasm_bindgen(typescript_type = "NomadConfig")]
    pub type NomadConfig;
}
