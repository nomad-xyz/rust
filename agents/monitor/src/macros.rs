// bails a restartable task
#[macro_export]
macro_rules! bail_task_if {
    ($cond:expr, $self:ident, $err:expr,) => {
        $crate::bail_task_if!($cond, $self, $err)
    };
    ($cond:expr, $self:ident, $err:expr) => {
        if $cond {
            let err = eyre::eyre!($err);
            tracing::error!(task = %$self, err = %err, "Task failed");
            return $crate::steps::TaskResult::Recoverable{task: $self, err};
        }
    };
}

#[macro_export]
macro_rules! unwrap_channel_item {
    ($channel_item:ident, $self:ident,) => {{
        unwrap_channel_item!($channel_item, $self)
    }};
    ($channel_item:ident, $self:ident) => {{
        if $channel_item.is_none() {
            tracing::debug!(
                task = %$self, "inbound channel broke"
            );
            return $crate::steps::TaskResult::Unrecoverable{err: eyre::eyre!("inbound channel broke"), worth_logging: false}
        }
        $channel_item.unwrap()
    }};
}

#[macro_export]
macro_rules! unwrap_or_bail {
    ($result:ident, $self:ident,) => {{
        unwrap_err_or_bail!($result, $self)
    }};
    ($result:ident, $self:ident) => {{
        bail_task_if!($result.is_err(), $self, $result.unwrap_err());
        $result.unwrap()
    }};
}
