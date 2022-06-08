#[macro_export]
/// Get remote networks from env
macro_rules! get_remotes_from_env {
    ($home:ident, $config:ident) => {{
        let connections = $config
            .protocol()
            .networks
            .get(&$home)
            .expect("!networks")
            .connections
            .clone();

        let all_connections = if let Ok(val) = std::env::var("AGENT_REPLICAS_ALL") {
            // If set to true/false, return value
            val.parse::<bool>().expect("misformatted AGENT_REPLICAS_ALL")
        } else {
            // If unset, return false
            false
        };

        if all_connections {
            connections
        } else {
            let mut remotes = std::collections::HashSet::new();
            for i in 0.. {
                let replica_var = format!("AGENT_REPLICA_{}_NAME", i);
                let replica_res = std::env::var(&replica_var);

                if let Ok(replica) = replica_res {
                    if connections.get(&replica).is_none() {
                        panic!("Attempted to run agent with unconnected replica. Home: {}. Replica: {}", $home, &replica);
                    }

                    remotes.insert(replica);
                } else {
                    break;
                }
            }

            remotes
        }
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
                    let remote_networks = nomad_base::get_remotes_from_env!(home, config);
                    color_eyre::eyre::ensure!(!remote_networks.is_empty(), "Must pass in at least one replica through env");
                    tracing::info!(remote_networks = ?remote_networks, "Loading settings for remote networks.");

                    let mut all_networks = remote_networks.clone();
                    all_networks.insert(home.clone());

                    // Get agent secrets
                    let secrets = nomad_xyz_configuration::AgentSecrets::from_env(&all_networks).expect("failed to build AgentSecrets from env");
                    secrets.validate(&agent, &all_networks)?;

                    // Create base settings
                    let base = nomad_base::Settings::from_config_and_secrets(&agent, &home, &remote_networks, &config, &secrets);
                    base.validate_against_config_and_secrets(&agent, &home, &remote_networks, &config, &secrets)?;

                    let mut agent = config.agent().get(&home).expect("agent config").[<$name:lower>].clone();

                    // Override with environment vars, if present
                    agent.load_env_overrides();

                    Ok(Self {
                        base,
                        agent,
                    })
                }
            }
        }
    };
}
