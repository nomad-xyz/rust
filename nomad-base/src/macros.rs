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
        impl AsMut<nomad_base::AgentCore> for $agent {
            fn as_mut(&mut self) -> &mut nomad_base::AgentCore {
                &mut self.core
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
        affix::paste! {
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
