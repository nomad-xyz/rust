// bails a restartable task
#[macro_export]
macro_rules! task_bail_if {
    ($cond:expr, $self:ident, $err:expr) => {
        if $cond {
            let err = eyre::eyre!($err);
            tracing::error!(task = ?$self, err = %err, "Task failed");
            return ($self, err);
        }
    };
}
