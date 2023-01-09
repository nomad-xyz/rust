use crate::k8s::ResultPodRestartStatus;
use crate::k8s::K8S;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;
use warp::Rejection;

use warp::http::StatusCode;
use warp::reply::with_status;

use regex::Regex;

const PORT: u16 = 3030;

enum RestartableAgent {
    Updater,
    Relayer,
    Processor,
}

impl FromStr for RestartableAgent {
    type Err = ();
    fn from_str(s: &str) -> Result<RestartableAgent, ()> {
        match s {
            "updater" => Ok(RestartableAgent::Updater),
            "relayer" => Ok(RestartableAgent::Relayer),
            "processor" => Ok(RestartableAgent::Processor),
            _ => Err(()),
        }
    }
}

impl ToString for RestartableAgent {
    fn to_string(&self) -> String {
        match self {
            RestartableAgent::Updater => "updater".to_string(),
            RestartableAgent::Relayer => "relayer".to_string(),
            RestartableAgent::Processor => "processor".to_string(),
        }
    }
}

struct Network(String);

impl FromStr for Network {
    type Err = ();
    fn from_str(s: &str) -> Result<Network, ()> {
        if Regex::new(r"^[a-z0-9]+$").unwrap().is_match(s) {
            Ok(Network(s.to_string()))
        } else {
            Err(())
        }
    }
}

impl ToString for Network {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}

fn with_k8s(
    k8s: Arc<Mutex<K8S>>,
) -> impl Filter<Extract = (Arc<Mutex<K8S>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || k8s.clone())
}

async fn handler(
    network: Network,
    agent: RestartableAgent,
    k8s: Arc<Mutex<K8S>>,
) -> Result<impl warp::Reply, Rejection> {
    let k8s = k8s.lock().await;
    if let Ok(status) = k8s.kill_pod(&network.to_string(), &agent.to_string()).await {
        let status_code = match status {
            ResultPodRestartStatus::Running => StatusCode::OK,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Ok(with_status(warp::reply::json(&status), status_code))
    } else {
        Err(warp::reject::not_found())
    }
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let k8s = K8S::new().await?;
    let k8s = Arc::new(Mutex::new(k8s));

    // Get ready for a <=30 sec timeout due to agent restarting.
    // I keep the connection open, to prevent complex logic for now
    let lifeguard = warp::path!("lifeguard" / Network / RestartableAgent)
        .and(with_k8s(k8s))
        .and_then(handler);

    warp::serve(lifeguard).run(([127, 0, 0, 1], PORT)).await;
    Ok(())
}
