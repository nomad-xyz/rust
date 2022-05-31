/// Get specified env var with format network_var OR get default_var if
/// network-specific not present.
pub fn network_or_default_from_env(network: &str, var: &str) -> Option<String> {
    let mut value = std::env::var(&format!(
        "{}_{}",
        network.to_uppercase(),
        var.to_uppercase()
    ))
    .ok();

    if value.is_none() {
        value = std::env::var(&format!("DEFAULT_{}", var.to_uppercase())).ok();
    }

    value
}
