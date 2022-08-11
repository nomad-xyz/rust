// bails a restartable task
#[macro_export]
macro_rules! bail_task_if {
    ($cond:expr, $self:ident, $err:expr,) => {
        $crate::bail_task_if!($cond, $self, $err)
    };
    ($cond:expr, $self:ident, $err:expr) => {
        if $cond {
            let err = eyre::eyre!($err);
            return $crate::TaskResult::Recoverable { task: $self, err };
        }
    };
}

#[macro_export]
macro_rules! unwrap_channel_item_unrecoverable {
    ($channel_item:ident, $self:ident,) => {{
        unwrap_channel_item_unrecoverable!($channel_item, $self)
    }};
    ($channel_item:ident, $self:ident) => {{
        if $channel_item.is_none() {
            tracing::debug!(
                task = %$self, "inbound channel broke"
            );
            return $crate::TaskResult::Unrecoverable{err: eyre::eyre!("inbound channel broke"), worth_logging: false}
        }
        $channel_item.unwrap()
    }};
}

#[macro_export]
macro_rules! unwrap_pipe_item_unrecoverable {
    ($pipe_item:ident, $self:ident,) => {{
        unwrap_pipe_item_unrecoverable!($pipe_item, $self)
    }};
    ($pipe_item:ident, $self:ident) => {{
        if $pipe_item.is_err() {
            tracing::debug!(
                task = %$self, "inbound pipe broke"
            );
            return $crate::TaskResult::Unrecoverable{err: eyre::eyre!("inbound pipe broke"), worth_logging: false}
        }
        $pipe_item.unwrap()
    }};
}

#[macro_export]
macro_rules! unwrap_result_recoverable {
    ($result:ident, $self:ident,) => {{
        unwrap_err_or_bail!($result, $self)
    }};
    ($result:ident, $self:ident) => {{
        bail_task_if!($result.is_err(), $self, $result.unwrap_err());
        $result.unwrap()
    }};
}

#[macro_export]
macro_rules! send_unrecoverable {
    ($tx:expr, $item:expr, $self:ident) => {
        if $tx.send($item).is_err() {
            return $crate::TaskResult::Unrecoverable {
                err: eyre::eyre!("outbound channel broke"),
                worth_logging: false,
            };
        }
    };
}
