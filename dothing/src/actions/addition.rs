use std::collections::HashMap;

use docker_api::opts::{ContainerConnectionOpts, ContainerCreateOpts};
use serde_json::Value;

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
        fn map_to_vec_string(map: &HashMap<String, Value>) -> Vec<String> {
            let mut result = Vec::new();

            for (key, value) in map {
                let formatted_string = format!("{}={}", key, value.as_str().unwrap());
                result.push(formatted_string);
            }

            result
        }

        info!("Processing env_vars from Hash to Vec");

        let vars = map_to_vec_string(&self.env_vars);

        info!("env_vars processed");

        info!("Building ContainerCreationOptions...");

        let opts = ContainerCreateOpts::builder()
            .image(&self.image)
            .env(vars)
            .command(&self.args)
            .build();

        info!("Built ContainerCreationOptions");

        opts
    }
}
