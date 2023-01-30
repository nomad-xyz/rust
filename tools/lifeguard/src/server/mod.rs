pub mod backoff;
pub mod errors;
pub mod handlers;
pub mod params;

use self::params::Network;
use self::params::RestartableAgent;
use crate::k8s::K8S;
use crate::metrics::metrics::Metrics;
use crate::server::handlers::*;

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{error, instrument, warn};
use warp::Filter;

const PORT: u16 = 3030;

/// Function that produces a warp `Filter` which offers `K8S` structure to a response handler
fn with_k8s(
    k8s: Arc<Mutex<K8S>>,
) -> impl Filter<Extract = (Arc<Mutex<K8S>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || k8s.clone())
}

/// Function that produces a warp `Filter` which offers `Metrics` to a response handler
fn with_metrics(
    metrics: Arc<Metrics>,
) -> impl Filter<Extract = (Arc<Metrics>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || metrics.clone())
}

/// Function that runs the server:
///   * instantiates `K8S` and `Metrics`
///   * creates warp routs
///   * starts both, the main and the metrics server
#[instrument]
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = Arc::new(Metrics::new(
        Some(3031),
        Arc::new(prometheus::Registry::new()),
    )?);
    let k8s = Arc::new(Mutex::new(K8S::new(metrics.clone()).await?));

    let lifeguard_route = warp::any()
        .and(warp::path("lifeguard"))
        .and(warp::path::param::<Network>())
        .and(warp::path::param::<RestartableAgent>())
        .and(with_k8s(k8s))
        .and(with_metrics(metrics.clone()));

    let restart_route = lifeguard_route
        .clone()
        .and(warp::post())
        .and(warp::path("restart"))
        .and(warp::path::end())
        .and_then(restart_handler);

    let status_route = lifeguard_route
        .and(warp::get())
        .and(warp::path("status"))
        .and(warp::path::end())
        .and_then(status_handler);

    let healthcheck_endpoint = warp::get()
        .and(warp::path!("healthcheck"))
        .and(warp::path::end())
        .and(with_metrics(metrics.clone()))
        .and_then(healthcheck);

    let routes = warp::any()
        .and(healthcheck_endpoint)
        .or(restart_route)
        .or(status_route)
        .recover(handle_rejection);

    let server_handle =
        tokio::spawn(async move { warp::serve(routes).run(([127, 0, 0, 1], PORT)).await });
    let metrics_handle = metrics.run_http_server();

    tokio::select! {
        server_result = server_handle => {
            if let Err(error) = server_result {
                error!(error = ?error, "Main server finished unexpectedly")
            } else {
                warn!("Main server finished, exiting...")
            }
        },
        metrics_result = metrics_handle => {
            if let Err(error) = metrics_result {
                error!(error = ?error, "Metrics server finished unexpectedly")
            } else {
                warn!("Metrics server finished, exiting...")
            }
        },
    }
    Ok(())
}
