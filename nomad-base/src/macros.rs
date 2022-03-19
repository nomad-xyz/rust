#[macro_export]
/// Shortcut for aborting a joinhandle and then awaiting and discarding its result
macro_rules! cancel_task {
    ($task:ident) => {
        #[allow(unused_must_use)]
        {
            let t = $task.into_inner();
            t.abort();
            t.await;
        }
    };
}

#[macro_export]
/// Shortcut for implementing agent traits
macro_rules! impl_as_ref_core {
    ($agent:ident) => {
        impl AsRef<nomad_base::AgentCore> for $agent {
            fn as_ref(&self) -> &nomad_base::AgentCore {
                &self.core
            }
        }
    };
}

#[macro_export]
/// Declare a new agent struct with the additional fields
macro_rules! decl_agent {
    (
        $(#[$outer:meta])*
        $name:ident{
            $($prop:ident: $type:ty,)*
        }) => {

        $(#[$outer])*
        #[derive(Debug)]
        pub struct $name {
            $($prop: $type,)*
            core: nomad_base::AgentCore,
        }

        $crate::impl_as_ref_core!($name);
    };
}

#[macro_export]
/// Declare a new channel block
/// ### Usage
///
/// ```ignore
/// decl_channel!(Relayer {
///     updates_relayed_counts: prometheus::IntCounterVec,
///     interval: u64,
/// });

/// ```
macro_rules! decl_channel {
    (
        $name:ident {
            $($(#[$tags:meta])* $prop:ident: $type:ty,)*
        }
    ) => {
        paste::paste! {
            #[derive(Debug, Clone)]
            #[doc = "Channel for `" $name]
            pub struct [<$name Channel>] {
                pub(crate) base: nomad_base::ChannelBase,
                $(
                    $(#[$tags])*
                    pub(crate) $prop: $type,
                )*
            }

            impl AsRef<nomad_base::ChannelBase> for [<$name Channel>] {
                fn as_ref(&self) -> &nomad_base::ChannelBase {
                    &self.base
                }
            }

            impl [<$name Channel>] {
                pub fn home(&self) -> Arc<CachingHome> {
                    self.as_ref().home.clone()
                }

                pub fn replica(&self) -> Arc<CachingReplica> {
                    self.as_ref().replica.clone()
                }

                pub fn db(&self) -> nomad_base::NomadDB {
                    self.as_ref().db.clone()
                }
            }
        }
    }
}

/// Trait agent-specific settings must implement to be constructed from
/// NomadConfig and AgentSecrets blocks. Used in decl_settings! macro.
pub trait AgentSettingsBlock {
    fn from_config_and_secrets(
        home_network: &str,
        config: &nomad_xyz_configuration::NomadConfig,
        secrets: &crate::settings::AgentSecrets,
    ) -> Self;
}

#[macro_export]
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
