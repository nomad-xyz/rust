use std::fmt;

use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use kube::api::DeleteParams;
use kube::core::WatchEvent;
use kube::{
    api::{Api, ListParams, ResourceExt},
    Client,
};

const ENVIRONMENT: &str = "dev";

#[derive(PartialEq, Debug)]
pub enum ResultPodRestartStatus {
    Deleted,
    Created,
    Running,
    Timeout,
    NotFound,
    None,
}

impl fmt::Display for ResultPodRestartStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResultPodRestartStatus::Deleted => write!(f, "Deleted"),
            ResultPodRestartStatus::Created => write!(f, "Created"),
            ResultPodRestartStatus::Running => write!(f, "Running"),
            ResultPodRestartStatus::Timeout => write!(f, "Timeout"),
            ResultPodRestartStatus::NotFound => write!(f, "NotFound"),
            ResultPodRestartStatus::None => write!(f, "None"),
        }
    }
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

        let pod = if let Some(pod) = pods.get_opt(name).await? {
            pod
        } else {
            return Ok(ResultPodRestartStatus::NotFound);
        };

        println!("Found! -> {}", pod.name_any());
        pods.delete(name, &DeleteParams::default()).await?;

        let mut latest_status = ResultPodRestartStatus::None;

        let lp = ListParams::default()
            .fields(&format!("metadata.name={}", name))
            .timeout(30);
        let mut stream = pods.watch(&lp, "0").await?.boxed();

        // TODO: clean this \/
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
                    // We probably don't really need `pod_recreated`
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

        if latest_status == ResultPodRestartStatus::None {
            latest_status = ResultPodRestartStatus::Timeout;
        }
        println!("Done! -> {}", pod_running);

        Ok(latest_status)
    }
}
