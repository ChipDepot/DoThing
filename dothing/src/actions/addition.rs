use docker_api::opts::{ContainerConnectionOpts, ContainerCreateOpts};

use crate::dckr::{ConnectionBuilder, ContainerBuilder};

use starduck::AdditionOrder;

impl ConnectionBuilder for AdditionOrder {
    fn build_connection<I: AsRef<str>>(&self, container_id: I) -> ContainerConnectionOpts {
        ContainerConnectionOpts::builder(container_id)
            .network_id(&self.network_name)
            .build()
    }
}

impl ContainerBuilder for AdditionOrder {
    fn build_container(&self) -> ContainerCreateOpts {
        ContainerCreateOpts::builder()
            .image(&self.image)
            .env(&self.env_vars)
            .command(&self.args)
            .build()
    }
}
