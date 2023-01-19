mod k8s;
mod server;
use tracing_subscriber;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    server::run_server().await?;
    Ok(())
}
