/// Get specified env var with format network_var OR get default_var if
/// network-specific not present.
pub fn network_or_default_from_env(network: &str, var: &str) -> Option<String> {
    let mut rpc_style = std::env::var(&format!("{}_{}", network, var)).ok();
    if rpc_style.is_none() {
        rpc_style = std::env::var(&format!("DEFAULT_{}", var)).ok();
    }

    rpc_style
}
