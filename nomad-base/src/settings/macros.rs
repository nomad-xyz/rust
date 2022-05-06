#[macro_export]
/// Get remote networks from env
macro_rules! get_remotes_from_env {
    () => {{
        let mut remotes = std::collections::HashSet::new();
        for i in 0.. {
            let replica_var = format!("AGENT_REPLICA_{}_NAME", i);
            let replica_res = std::env::var(&replica_var);

            if let Ok(replica) = replica_res {
                remotes.insert(replica);
            } else {
                break;
            }
        }

        remotes
    }};
}

#[macro_export]
/// Declare a new agent settings block with base settings + agent-specific
/// settings
/// ### Usage
///
/// ```ignore
/// decl_settings!(Relayer, RelayerConfig,);
/// ```
macro_rules! decl_settings {
    ($name:ident, $agent_settings:ty) => {
        decl_settings!($name, $agent_settings,);
    };
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
                    // Get agent and home names
                    let agent = std::stringify!($name).to_lowercase();
                    let home = std::env::var("AGENT_HOME_NAME").expect("missing AGENT_HOME_NAME");

                    // Get config
                    let config_path = std::env::var("CONFIG_PATH").ok();
                    let config: nomad_xyz_configuration::NomadConfig = match config_path {
                        Some(path) => nomad_xyz_configuration::NomadConfig::from_file(path).expect("!config"),
                        None => {
                            let env = std::env::var("RUN_ENV").expect("missing RUN_ENV");
                            nomad_xyz_configuration::get_builtin(&env).expect("!config").to_owned()
                        }
                    };
                    config.validate()?;

                    // Get agent remotes
                    let all_remotes_res = std::env::var("AGENT_REPLICAS_ALL");
                    let remote_networks = if let Ok(all_remotes) = all_remotes_res {
                        config
                            .protocol()
                            .networks
                            .get(&home)
                            .expect("!networks")
                            .connections
                            .clone()
                    } else {
                        let remote_networks = nomad_base::get_remotes_from_env!();
                        color_eyre::eyre::ensure!(!remote_networks.is_empty(), "Must pass in at least one replica through env");
                        remote_networks
                    };

                    let mut all_networks = remote_networks.clone();
                    all_networks.insert(home.clone());

                    // Get agent secrets
                    let secrets = nomad_xyz_configuration::AgentSecrets::from_env(&all_networks).expect("failed to build AgentSecrets from env");
                    secrets.validate(&agent, &home, &remote_networks)?;

                    // Create base settings
                    let base = nomad_base::Settings::from_config_and_secrets(&agent, &home, &remote_networks, &config, &secrets);
                    base.validate_against_config_and_secrets(&agent, &home, &remote_networks, &config, &secrets)?;

                    let agent = config.agent().get(&home).expect("agent config").[<$name:lower>].clone();

                    Ok(Self {
                        base,
                        agent,
                    })
                }
            }
        }
    };
}
