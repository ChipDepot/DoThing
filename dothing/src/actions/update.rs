use crate::dckr::FindContainer;

use docker_api::opts::ContainerListOpts;
use docker_api::{Container, Docker};

use serde::Deserialize;
use uuid::Uuid;

use starduck::{QueryType, ReconfigureOrder};

#[async_trait::async_trait]
impl FindContainer for ReconfigureOrder {
    async fn find_container(&self, docker: &Docker) -> Option<Container> {
        let list_opts = ContainerListOpts::builder().all(true).build();
        let all_containers_sum = docker.containers().list(&list_opts).await.unwrap();

        for cont_sum in all_containers_sum {
            let id = cont_sum.id.unwrap();

            if let Ok(container_info) = docker.containers().get(&id).inspect().await {
                let url = match (container_info.name, &self.query_type) {
                    (Some(name), QueryType::Http { endpoint, port }) => {
                        format!("http://{}:{}{}", name, port, endpoint.to_string_lossy())
                    }
                    (_, _) => continue,
                };

                match Self::request_device_uuid(&url).await {
                    Some(uuid) if uuid == self.uuid.unwrap() => {
                        return Some(docker.containers().get(&id));
                    }
                    _ => continue,
                };
            }
        }

        None
    }

    async fn request_device_uuid(url: &str) -> Option<Uuid> {
        #[derive(Deserialize)]
        struct DeviceInfo {
            device_uuid: Uuid,
        }

        if let Ok(response) = reqwest::get(url).await.ok()?.json::<DeviceInfo>().await {
            return Some(response.device_uuid);
        }

        None
    }
}
