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

        let all_connections = std::env::var("AGENT_REPLICAS_ALL")
            .map_or(Ok(false), |val| val.parse::<bool>())
            .expect("misformatted AGENT_REPLICAS_ALL, expected 'true' or 'false'");

        if all_connections {
            tracing::info!(
                count = connections.len(),
                "All remotes configured",
            );
            connections
        } else {
            let connections = (0..)
                .map(|i| format!("AGENT_REPLICA_{}_NAME", i))
                .map(|s| std::env::var(&s))
                .take_while(Result::is_ok)
                .map(Result::unwrap)
                .map(|replica| {
                    if !connections.contains(&replica) {
                        panic!("Attempted to run agent with unconnected replica. Home: {}. Replica: {}", $home, &replica);
                    }
                    replica
                })
                .collect::<std::collections::HashSet<_>>();
                tracing::info!(
                    count = connections.len(),
                    "Remotes configured by env",
                );
                connections
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
            #[doc = "Settings for `" $name "`"]
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
                pub async fn new() -> color_eyre::Result<Self>{
                    // Get agent and home names
                    tracing::info!("Building settings from env");
                    let agent = std::stringify!($name).to_lowercase();
                    let home = std::env::var("AGENT_HOME_NAME").expect("missing AGENT_HOME_NAME");

                    // Get config
                    let config = if let Some(config_url) = std::env::var("CONFIG_URL").ok() {
                        tracing::info!(config_url = config_url.as_str(), "Loading config from URL");
                        nomad_xyz_configuration::NomadConfig::fetch(&config_url).await.expect("!config url")
                    } else if let Some(config_path) = std::env::var("CONFIG_PATH").ok() {
                        tracing::info!(config_path = config_path.as_str(), "Loading config from file");
                        nomad_xyz_configuration::NomadConfig::from_file(config_path).expect("!config path")
                    } else if let Some(env) = std::env::var("RUN_ENV").ok() {
                        tracing::info!(env = env.as_str(), "Loading config from built-in");
                        nomad_xyz_configuration::get_builtin(&env).expect("!config builtin").to_owned()
                    } else {
                        color_eyre::eyre::bail!("No configuration found. Set CONFIG_URL or CONFIG_PATH or RUN_ENV")
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
