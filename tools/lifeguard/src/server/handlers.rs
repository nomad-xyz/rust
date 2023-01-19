use crate::k8s::{LifeguardPod, K8S};
use crate::server::errors::ServerRejection;
use crate::server::params::{Network, RestartableAgent};
use k8s_openapi::http::StatusCode;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::reply::with_status;
use warp::{Rejection, Reply};

pub async fn restart_handler(
    network: Network,
    agent: RestartableAgent,
    k8s: Arc<Mutex<K8S>>,
) -> Result<impl warp::Reply, Rejection> {
    let pod = LifeguardPod::new(network, agent);
    println!("RESTART HANDLER");

    let k8s = k8s.lock().await;

    if let Err(error) = k8s.drop_pod(&pod).await.into() {
        let rejection: ServerRejection = error.into();
        Err(rejection.into())
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
    let pod = LifeguardPod::new(network, agent);
    let k8s = k8s.lock().await;
    match k8s.status(&pod).await {
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
