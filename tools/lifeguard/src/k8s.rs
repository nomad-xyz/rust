use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use kube::api::DeleteParams;
use kube::core::WatchEvent;
use kube::{
    api::{Api, ListParams, ResourceExt},
    Client,
};
use serde::Serialize;

use crate::server::params::{Network, RestartableAgent};

const ENVIRONMENT: &str = "dev";

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
pub struct PodStatus {
    start_time: Option<Time>,
    phase: String,
}

// use std::{error::Error, fmt};

#[derive(Debug)]
enum K8sError {
    Random,
}

impl std::error::Error for K8sError {}

impl std::fmt::Display for K8sError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

#[derive(PartialEq, Debug, Serialize)]
#[serde(tag = "status")]
pub enum ResultPodRestartStatus {
    Deleted,
    Created,
    Running,
    Timeout,
    NotFound,
}

pub struct K8S {
    client: Client,
}

impl K8S {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::try_default().await?;
        Ok(K8S { client })
    }

    pub async fn kill_pod(
        &self,
        network: &str,
        agent_name: &str,
    ) -> Result<ResultPodRestartStatus, Box<dyn std::error::Error>> {
        let pods: Api<Pod> = Api::default_namespaced(self.client.clone());

        let name = &format!("{}-{}-{}-0", ENVIRONMENT, network, agent_name);
        println!("Supposed to kill this one: {}", name);

        if let Some(pod) = pods.get_opt(name).await? {
            println!("Found requested pod: {}!", pod.name_any());
        } else {
            return Ok(ResultPodRestartStatus::NotFound);
        };

        pods.delete(name, &DeleteParams::default()).await?;

        let lp = ListParams::default()
            .fields(&format!("metadata.name={}", name))
            .timeout(30);
        let mut stream = pods.watch(&lp, "0").await?.boxed();

        // TODO: clean this \/
        let mut latest_status = ResultPodRestartStatus::Timeout;
        let mut pod_deleted = false;
        let mut pod_recreated = false;
        let mut pod_running = false;
        while let Some(event) = stream.try_next().await? {
            match event {
                WatchEvent::Added(pod) => {
                    if pod_deleted {
                        pod_recreated = true;
                        latest_status = ResultPodRestartStatus::Created;
                    }
                    println!("ADDED: {}", pod.name_any())
                }
                WatchEvent::Modified(pod) => {
                    println!(
                        "UPDATED: {}->{:?}",
                        pod.name_any(),
                        pod.status.as_ref().and_then(|s| s.phase.as_ref())
                    );

                    if pod.status.and_then(|status| status.phase).as_deref() == Some("Running")
                        && (pod_recreated || pod_deleted)
                    // We probably don't really need `pod_recreated`, so I omit the logic for now
                    {
                        pod_running = true;
                        println!("RUNNING!");
                        latest_status = ResultPodRestartStatus::Running;
                        break;
                    }
                }
                WatchEvent::Deleted(pod) => {
                    pod_deleted = true;
                    latest_status = ResultPodRestartStatus::Deleted;
                    println!("DELETED: {}", pod.name_any())
                }
                WatchEvent::Error(e) => println!("ERROR: {} {} ({})", e.code, e.message, e.status),
                _ => {}
            };
        }

        println!("Done! -> {}", pod_running);

        Ok(latest_status)
    }

    pub async fn delete_pod(&self, pod: &LifeguardPod) -> Result<(), Box<dyn std::error::Error>> {
        let pods: Api<Pod> = Api::default_namespaced(self.client.clone());

        // let name = &format!("{}-{}-{}-0", ENVIRONMENT, network, agent_name);
        let name = pod.to_string();

        pods.delete(&name, &DeleteParams::default()).await?;
        println!("Deleted pod: {}", name);

        Ok(())
    }

    pub async fn status_pod(
        &self,
        network: &str,
        agent_name: &str,
    ) -> Result<PodStatus, Box<dyn std::error::Error>> {
        let pods: Api<Pod> = Api::default_namespaced(self.client.clone());

        let name = &format!("{}-{}-{}-0", ENVIRONMENT, network, agent_name);

        if let Some(pod) = pods.get_opt(name).await? {
            println!("Found requested pod: {}!", pod.name_any());
            if let Some(status) = pod.status {
                let start_time = status.start_time;
                let phase = status.phase.unwrap();
                return Ok(PodStatus { start_time, phase });
            }
        }
        return Err(Box::new(K8sError::Random));
    }
}
