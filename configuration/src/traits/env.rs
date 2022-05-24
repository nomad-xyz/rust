/// Implemented by structs overridable through environment variables
pub trait EnvOverridable {
    /// Override self.fields through env vars
    fn load_env_overrides(&mut self);
}

/// Given optional prefix and network, concatenate values to create full prefix.
/// If both prefix and network provided, structure is
/// network_prefix_postfix. If no network provided, structure is
/// prefix_postfix. If no prefix, structure is network_postfix. Panic if no
/// network or prefix.
pub fn full_prefix(prefix: Option<&str>, network: Option<&str>) -> String {
    if let Some(prefix) = prefix {
        if let Some(network) = network {
            format!("{}_{}", network, prefix)
        } else {
            prefix.to_owned()
        }
    } else {
        if let Some(network) = network {
            return network.to_owned();
        }

        panic!("Cannot call from_env without providing a prefix or network");
    }
}
