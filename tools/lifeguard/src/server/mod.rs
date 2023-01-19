use crate::k8s::K8S;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

use self::backoff::RestartBackoff;
use self::params::Network;
use self::params::RestartableAgent;

pub mod backoff;
pub mod errors;
pub mod handlers;
pub mod params;
use crate::server::handlers::*;

const PORT: u16 = 3030;

fn with_k8s(
    k8s: Arc<Mutex<K8S>>,
) -> impl Filter<Extract = (Arc<Mutex<K8S>>,), Error = std::convert::Infallible> + Clone {
    println!("with_k8s");
    warp::any().map(move || k8s.clone())
}

fn with_backoff(
    backoff: Arc<Mutex<RestartBackoff>>,
) -> impl Filter<Extract = (Arc<Mutex<RestartBackoff>>,), Error = std::convert::Infallible> + Clone
{
    println!("with_RestartBackoff");
    warp::any().map(move || backoff.clone())
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let k8s = Arc::new(Mutex::new(K8S::new().await?));

    let lifeguard_route = warp::any()
        .and(warp::path("lifeguard"))
        .and(warp::path::param::<Network>())
        .and(warp::path::param::<RestartableAgent>())
        .and(with_k8s(k8s));

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
        .and_then(healthcheck);

    let debug = warp::path("de").and(warp::path("bug"));
    let debug_route = debug.and_then(healthcheck);

    let routes = warp::any()
        .and(healthcheck_endpoint)
        .or(restart_route)
        .or(status_route)
        .or(debug_route)
        .recover(handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], PORT)).await;
    Ok(())
}
