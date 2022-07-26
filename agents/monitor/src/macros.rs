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
            return ($self, err);
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
        }
        $channel_item.expect("inbound channel broke")
    }};
}
