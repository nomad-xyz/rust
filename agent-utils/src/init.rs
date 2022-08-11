use nomad_xyz_configuration::{get_builtin, NomadConfig};
use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn config_from_file() -> Option<NomadConfig> {
    std::env::var("CONFIG_PATH")
        .ok()
        .and_then(|path| NomadConfig::from_file(path).ok())
}

pub fn config_from_env() -> Option<NomadConfig> {
    std::env::var("RUN_ENV")
        .ok()
        .and_then(|env| get_builtin(&env))
        .map(ToOwned::to_owned)
}

pub fn config() -> eyre::Result<NomadConfig> {
    config_from_file()
        .or_else(config_from_env)
        .ok_or_else(|| eyre::eyre!("Unable to load config from file or env"))
}

pub fn init_tracing() {
    let builder = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env())
        .with_level(true);
    if std::env::var("MONITOR_PRETTY").is_ok() {
        builder.pretty().init()
    } else {
        builder.json().init()
    }
}

pub fn networks_from_env() -> Option<Vec<String>> {
    std::env::var("MONITOR_NETWORKS")
        .ok()
        .map(|s| s.split(',').map(ToOwned::to_owned).collect())
}

pub fn rpc_from_env(network: &str) -> Option<String> {
    std::env::var(format!("{}_CONNECTION_URL", network.to_uppercase())).ok()
}
