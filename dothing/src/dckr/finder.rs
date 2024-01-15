use docker_api::{Container, Docker};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait FindContainer {
    async fn find_container(&self, docker: &Docker) -> Option<Container>;
    async fn request_device_uuid(url: &str) -> Option<Uuid>;
}
