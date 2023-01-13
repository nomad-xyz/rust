use crate::k8s::{LifeguardPod, K8S};
use crate::server::errors::{ErrorMessage, ServerRejection};
use crate::server::params::{Network, RestartableAgent};
use k8s_openapi::http::StatusCode;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::reply::with_status;
use warp::{Rejection, Reply};

use super::backoff::RestartBackoff;

pub async fn restart_handler(
    network: Network,
    agent: RestartableAgent,
    k8s: Arc<Mutex<K8S>>,
    backoff: Arc<Mutex<RestartBackoff>>,
) -> Result<impl warp::Reply, Rejection> {
    let pod = LifeguardPod::new(network, agent);
    println!("RESTART HANDLER");
    // return Ok(warp::reply());
    let mut backoff = backoff.lock().await;
    if !backoff.inc(&pod) {
        return Err(warp::reject::custom(ServerRejection::PodLimitElapsed));
    }

    return Ok(warp::reply());

    let k8s = k8s.lock().await;

    if let Err(e) = k8s.delete_pod(&pod).await {
        Err(warp::reject::custom(ServerRejection::InternalError(
            e.to_string(),
        )))
    } else {
        Ok(warp::reply())
    }
}

pub async fn status_handler(
    network: Network,
    agent: RestartableAgent,
    k8s: Arc<Mutex<K8S>>,
) -> Result<impl warp::Reply, Rejection> {
    println!("Status handler");
    let k8s = k8s.lock().await;
    match k8s
        .status_pod(&network.to_string(), &agent.to_string())
        .await
    {
        Ok(status) => Ok(with_status(warp::reply::json(&status), StatusCode::OK)),
        Err(e) => Err(warp::reject::custom(ServerRejection::InternalError(
            e.to_string(),
        ))),
    }
}

pub async fn healthcheck() -> Result<impl warp::Reply, Rejection> {
    Ok(warp::reply())
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    println!("handle err----> {:?}", err);
    // TODO options
    let mut code = StatusCode::INTERNAL_SERVER_ERROR;
    let mut message = "";
    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if let Some(server_rejection) = err.find::<ServerRejection>() {
        match server_rejection {
            ServerRejection::InternalError(s) => message = s,
            ServerRejection::Status(s) => code = *s,
            ServerRejection::PodLimitElapsed => {
                code = StatusCode::TOO_MANY_REQUESTS;
                message = "Too many restarts of the pod"
            }
        }
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
