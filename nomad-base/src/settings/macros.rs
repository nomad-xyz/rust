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
                pub fn new() -> color_eyre::Result<Self>{
                    let agent = std::stringify!($name).to_lowercase();
                    let env = std::env::var("RUN_ENV").expect("missing RUN_ENV env var");
                    let home = std::env::var("AGENT_HOME").expect("missing AGENT_HOME env var");
                    let secrets_path = std::env::var("SECRETS_PATH").ok();
                    let config_path = std::env::var("CONFIG_PATH").ok();

                    let config: nomad_xyz_configuration::NomadConfig = match config_path {
                        Some(path) => {
                            let file = std::fs::File::open(&path)?;
                            let reader = std::io::BufReader::new(file);
                            serde_json::from_reader(reader).expect("json malformed")
                        }
                        None => nomad_xyz_configuration::get_builtin(&env).expect("!config").to_owned(),
                    };
                    config.validate()?;

                    let secrets = match secrets_path {
                        Some(path) =>  nomad_xyz_configuration::AgentSecrets::from_file(path).expect("failed to build AgentSecrets from file"),
                        None => nomad_xyz_configuration::AgentSecrets::from_env("").expect("failed to build AgentSecrets from env"),
                    };
                    secrets.validate_against_config(&agent, &home, &config)?;

                    let base = nomad_base::Settings::from_config_and_secrets(&agent, &home, &config, &secrets);
                    base.validate_against_config_and_secrets(&agent, &home, &config, &secrets)?;

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
