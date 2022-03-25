#[macro_export]
/// Declare a new agent settings block with base settings + agent-specific
/// settings
/// ### Usage
///
/// ```ignore
/// decl_settings!(Relayer, RelayerConfig,);
/// ```
macro_rules! decl_settings {
    ($name:ident, $agent_settings:ty,) => {
        affix::paste! {
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
                    let secrets_path = std::env::var("SECRETS_PATH").unwrap_or("./secrets.json".to_owned()); // default to ./secrets.json
                    let config_path = std::env::var("CONFIG_PATH").unwrap_or("./config.json".to_owned()); // default to ./config.json

                    let config = nomad_xyz_configuration::NomadConfig::from_file(&config_path)?;
                    config.validate()?;

                    let secrets = nomad_base::AgentSecrets::from_file(&secrets_path)?;
                    secrets.validate(&agent)?;

                    let base = nomad_base::Settings::from_config_and_secrets(&agent, &home, &config, &secrets);
                    let agent = config.agent().get(&home).expect("agent config").[<$name:lower>].clone();

                    Ok(Self {
                        base,
                        agent,
                    })
                }
            }
        }
    }
}
