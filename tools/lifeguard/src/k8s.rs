use std::fmt::Debug;

use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::Pod;
use kube::api::DeleteParams;
use kube::api::{Api, ResourceExt};
use kube::Client;
use serde::Serialize;

use crate::server::backoff::RestartBackoff;
use crate::server::errors::ServerRejection;
use crate::server::params::{Network, RestartableAgent};

use tracing::{debug, info, instrument};

const ENVIRONMENT: &str = "dev";

#[derive(Debug)]
pub struct LifeguardPod {
    network: Network,
    agent: RestartableAgent,
}

impl LifeguardPod {
    pub fn new(network: Network, agent: RestartableAgent) -> Self {
        Self {
            network: network,
            agent: agent,
        }
    }
}

impl ToString for LifeguardPod {
    fn to_string(&self) -> String {
        format!("{}-{}-{}-0", ENVIRONMENT, self.network, self.agent)
    }
}

#[derive(Serialize)]
pub enum PodStatus {
    Running(DateTime<Utc>),
    Phase(String),
}

#[derive(Debug)]
pub enum K8sError {
    TooEarly(DateTime<Utc>),
    NoPod,
    NoStatus,
    NoStartTime,
    Custom(Box<dyn std::error::Error>),
}

impl std::error::Error for K8sError {}

impl std::fmt::Display for K8sError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TooEarly(t) => write!(f, "{}", t),
            Self::NoPod => write!(f, "NoPod"),
            Self::NoStatus => write!(f, "NoStatus"),
            Self::NoStartTime => write!(f, "NoStartTime"),
            Self::Custom(e) => write!(f, "Custom Error: {}", e),
        }
    }
}

impl From<K8sError> for ServerRejection {
    fn from(error: K8sError) -> Self {
        match error {
            K8sError::TooEarly(t) => ServerRejection::TooEarly(t),
            K8sError::NoPod => ServerRejection::InternalError("NoPod".into()),
            K8sError::NoStatus => ServerRejection::InternalError("NoStatus".into()),
            K8sError::NoStartTime => ServerRejection::InternalError("NoStartTime".into()),
            K8sError::Custom(e) => ServerRejection::InternalError(e.to_string()),
        }
    }
}

pub struct K8S {
    client: Client,
    backoff: RestartBackoff,
    start_time_limit: Duration,
}

impl K8S {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::try_default().await?;
        let backoff = RestartBackoff::new(5)?;
        Ok(K8S {
            client,
            backoff,
            start_time_limit: Duration::seconds(25), // 1 min
        })
    }

    #[instrument]
    pub async fn check_backoff(&self, pod: &LifeguardPod) -> Result<(), K8sError> {
        info!(pod = ?pod, "Checking backoff");
        if let Some(next_attempt_time) = self.backoff.inc(pod).await {
            return Err(K8sError::TooEarly(next_attempt_time));
        }
        Ok(())
    }

    #[instrument]
    pub async fn check_start_time(&self, pod: &LifeguardPod) -> Result<(), K8sError> {
        debug!(pod = ?pod, "Checking start time");
        if let PodStatus::Running(start_time) = self.status(pod).await? {
            let target_time = start_time + self.start_time_limit;
            if target_time > Utc::now() {
                return Err(K8sError::TooEarly(target_time));
            }
        }

        Ok(())
    }

    #[instrument]
    pub async fn delete_pod(&self, pod: &LifeguardPod) -> Result<(), K8sError> {
        debug!(pod = ?pod, "Started deleting pod");
        let pods: Api<Pod> = Api::default_namespaced(self.client.clone());
        let pod_name = pod.to_string();

        pods.delete(&pod_name, &DeleteParams::default())
            .await
            .map_err(|e| K8sError::Custom(Box::new(e)))?;
        info!(pod = ?pod, "Deleted pod");

        Ok(())
    }

    #[instrument]
    pub async fn drop_pod(&self, pod: &LifeguardPod) -> Result<(), K8sError> {
        debug!(pod = ?pod, "Starting full deleting pod procedure");
        // Should run in sequence
        self.check_start_time(pod).await?;
        self.check_backoff(pod).await?;

        self.delete_pod(pod).await?;
        debug!(pod = ?pod, "Finished full deleting pod procedure");
        Ok(())
    }

    #[instrument]
    pub async fn status(&self, pod: &LifeguardPod) -> Result<PodStatus, K8sError> {
        let pods: Api<Pod> = Api::default_namespaced(self.client.clone());

        let name = pod.to_string();

        if let Some(pod) = pods
            .get_opt(&name)
            .await
            .map_err(|e| K8sError::Custom(Box::new(e)))?
        {
            debug!(pod = ?pod, "Found requested pod");
            if let Some(status) = pod.status {
                let start_time = status.start_time.ok_or(K8sError::NoStatus)?;
                let phase = status.phase.ok_or(K8sError::NoStartTime)?;
                if phase == "Running" {
                    return Ok(PodStatus::Running(start_time.0));
                } else {
                    return Ok(PodStatus::Phase(phase));
                }
            }
        }
        return Err(K8sError::NoPod);
    }
}

impl Debug for K8S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("K8S")
            .field("backoff", &self.backoff)
            .field("start_time_limit", &self.start_time_limit)
            .finish()
    }
}
