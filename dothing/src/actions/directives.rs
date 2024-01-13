use docker_api::opts::ContainerConnectionOpts;
use starduck::Directive;

use crate::dckr::{ConnectionBuilder, ContainerBuilder};

impl ConnectionBuilder for Directive {
    fn build_connection<I>(&self, container_id: I) -> ContainerConnectionOpts
    where
        I: AsRef<str>,
    {
        let network = match &self {
            Directive::Addition { network, .. } | Directive::Reconfigure { network, .. } => network,
        };
        todo!()
    }
}
