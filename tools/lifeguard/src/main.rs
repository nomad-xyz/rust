mod k8s;
mod server;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    server::run_server().await?;
    Ok(())
}
