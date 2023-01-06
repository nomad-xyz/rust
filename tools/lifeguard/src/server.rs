use crate::k8s::ResultPodRestartStatus;
use crate::k8s::K8S;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use warp::Rejection;

use warp::http::StatusCode;
use warp::reply::with_status;

fn with_k8s(
    k8s: Arc<Mutex<K8S>>,
) -> impl Filter<Extract = (Arc<Mutex<K8S>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || k8s.clone())
}

async fn handler(
    network: String,
    agent: String,
    k8s: Arc<Mutex<K8S>>,
) -> Result<impl warp::Reply, Rejection> {
    let k8s = k8s.lock().await;
    if let Ok(status) = k8s.kill_pod(&network, &agent).await {
        let status_code = match status {
            ResultPodRestartStatus::Running => StatusCode::OK,
            ResultPodRestartStatus::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::GATEWAY_TIMEOUT,
        };
        Ok(with_status(status.to_string(), status_code))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let k8s = K8S::new().await?;
    let k8s = Arc::new(Mutex::new(k8s));

    // Get ready for a ~30 sec timeout
    let lifeguard = warp::path!("lifeguard" / String / String)
        .and(with_k8s(k8s))
        .and_then(handler);

    warp::serve(lifeguard).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
