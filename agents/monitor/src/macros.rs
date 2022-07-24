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
macro_rules! unwrap_pipe_item {
    ($pipe_output:ident, $self:ident,) => {{
        unwrap_pipe_output!($pipe_output, $self)
    }};
    ($pipe_output:ident, $self:ident) => {{
        $crate::bail_task_if!($pipe_output.is_err(), $self, $pipe_output.unwrap_err(),);

        let item_opt = $pipe_output.unwrap();
        $crate::bail_task_if!(item_opt.is_none(), $self, "inbound pipe failed",);

        item_opt.unwrap()
    }};
}
