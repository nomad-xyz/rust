/// Implemented by structs overridable through environment variables
pub trait EnvOverridable {
    /// Override self.fields through env vars
    fn load_env_overrides(&mut self);
}

/// Implemented by structs that are built from environment variables (signers,
/// connections, etc)
pub trait FromEnv {
    /// Optionally load self from env vars.
    /// Accepts a `default_prefix` which will be looked for if `prefix` isn't found.
    /// If both are present, `prefix` has precedence over `default_prefix`.
    /// Return None if *any* necessary env var is missing.
    fn from_env(prefix: &str, default_prefix: Option<&str>) -> Option<Self>
    where
        Self: Sized;
}
