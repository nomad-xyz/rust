/// Trait agent-specific settings must implement to be constructed from
/// NomadConfig and AgentSecrets blocks. Used in decl_settings! macro.
pub trait AgentSettingsBlock {
    /// Describe how to retrieve agent-specific settings from both
    /// NomadConfig and AgentSecrets blocks
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        secrets: &crate::settings::AgentSecrets,
    ) -> Self;
}

#[macro_export]
/// Declare a new agent settings block
/// ### Usage
///
/// ```ignore
/// #[derive(Debug, Clone, serde::Deserialize)]
/// pub struct RelayerSettingsBlock {
///    pub interval: u64,
/// }
///
/// impl AgentSettingsBlock for RelayerSettingsBlock {
///     fn from_config_and_secrets(
///         home_network: &str,
///         config: &nomad_xyz_configuration::NomadConfig,
///         _secrets: &AgentSecrets,
///     ) -> Self {
///         let interval = config.agent().get(home_network).unwrap().relayer.interval;
///         Self { interval }
///     }
/// }
///
/// decl_settings!(Relayer, RelayerSettingsBlock,);
/// ```
macro_rules! decl_settings {
    ($name:ident, $agent_settings:ty,) => {
        paste::paste! {
            #[derive(Debug, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            #[doc = "Settings for `" $name]
            pub struct [<$name Settings>] {
                #[serde(flatten)]
                pub(crate) base: nomad_base::Settings,
                pub(crate) agent: $agent_settings, // TODO: flatten out struct fields

            }

            impl AsRef<nomad_base::Settings> for [<$name Settings>] {
                fn as_ref(&self) -> &nomad_base::Settings {
                    &self.base
                }
            }

            impl [<$name Settings>] {
                pub fn new() -> Result<Self, color_eyre::Report>{
                    let agent = std::stringify!($name).to_lowercase();
                    let env = std::env::var("RUN_ENV").expect("missing RUN_ENV env var");
                    let home = std::env::var("AGENT_HOME").expect("missing AGENT_HOME env var");
                    let secrets_path = &std::env::var("SECRETS_PATH").expect("missing SECRETS_PATH env var");

                    let config = nomad_xyz_configuration::get_builtin(&env).expect("!config");
                    let secrets = nomad_base::AgentSecrets::from_file(secrets_path.into())?;

                    let base = nomad_base::Settings::from_config_and_secrets(&agent, &home, &config, &secrets);
                    let agent = <$agent_settings as AgentSettingsBlock>::from_config_and_secrets(&home, &config, &secrets);

                    Ok(Self {
                        base,
                        agent,
                    })
                }
            }
        }
    }
}
