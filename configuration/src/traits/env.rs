/// Implemented by structs overridable through environment variables
pub trait EnvOverridable {
    /// Override self.fields through env vars
    fn load_env_overrides(&mut self, require_all: bool);
}

/// Implemented by structs that are built from environment variables (signers, 
/// connections, etc)
pub trait FromEnv {
    /// Optionally load self from env vars. Return None if any necessary env var 
    /// is missing.
    fn from_env(prefix: &str) -> Option<Self>
    where
        Self: Sized;
}
