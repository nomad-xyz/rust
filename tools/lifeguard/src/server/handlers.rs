use crate::k8s::{LifeguardPod, K8S};
use crate::metrics::metrics::Metrics;
use crate::server::errors::ServerRejection;
use crate::server::params::{Network, RestartableAgent};

use std::convert::Infallible;
use std::sync::Arc;

use k8s_openapi::http::StatusCode;
use tokio::sync::Mutex;
use tracing::{debug, instrument};
use warp::reply::with_status;
use warp::{Rejection, Reply};

/// Warp handler which is called when requesting party wants to restart a pod
#[instrument]
pub async fn restart_handler(
    network: Network,
    agent: RestartableAgent,
    k8s: Arc<Mutex<K8S>>,
    metrics: Arc<Metrics>,
) -> Result<impl warp::Reply, Rejection> {
    metrics.incoming_requests_inc("restart", &network.to_string(), &agent.to_string());

    let pod = LifeguardPod::new(network.clone(), agent.clone());
    debug!(pod = ?pod, "Restart Handler");
    let k8s = k8s.lock().await;

    if let Err(error) = k8s.try_delete_pod(&pod).await.into() {
        let rejection: ServerRejection = error.into();
        Err(rejection.into())
    } else {
        metrics.restarts_inc(&network.to_string(), &agent.to_string());
        Ok(warp::reply())
    }
}

/// Warp handler which is called when requesting party wants to get a status of a pod
#[instrument]
pub async fn status_handler(
    network: Network,
    agent: RestartableAgent,
    k8s: Arc<Mutex<K8S>>,
    metrics: Arc<Metrics>,
) -> Result<impl warp::Reply, Rejection> {
    metrics.incoming_requests_inc("status", &network.to_string(), &agent.to_string());

    let pod = LifeguardPod::new(network, agent);
    debug!(pod = ?pod, "Status handler");
    let k8s = k8s.lock().await;
    match k8s.status(&pod).await {
        Ok(status) => Ok(with_status(warp::reply::json(&status), StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(ServerRejection::InternalError(
            e.to_string(),
        ))),
    }
}

/// Warp handler which is called when requesting party wants to get a healthcheck of a server
#[instrument]
pub async fn healthcheck(metrics: Arc<Metrics>) -> Result<impl warp::Reply, Rejection> {
    metrics.incoming_requests_inc("healthcheck", "", "");
    Ok(warp::reply())
}

/// Warp handler which is called if a server raised a rejection.
/// It produces a response body and a relevant status code
#[instrument]
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    debug!(err = ?err, "Handling rejection");
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            "".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(server_rejection) = err.find::<ServerRejection>() {
        let (code, body) = server_rejection.code_and_message();

        let json = serde_json::to_string(&body).unwrap();

        Ok(warp::reply::with_status(json, code))
    } else {
        Ok(warp::reply::with_status(
            "".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
