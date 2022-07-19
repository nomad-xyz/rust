// bails a restartable task
#[macro_export]
macro_rules! bail_task_if {
    ($cond:expr, $self:ident, $err:expr,) => {
        task_bail_if!($cond, $self, $err)
    };
    ($cond:expr, $self:ident, $err:expr) => {
        if $cond {
            let err = eyre::eyre!($err);
            tracing::error!(task = ?$self, err = %err, "Task failed");
            return ($self, err);
        }
    };
}
