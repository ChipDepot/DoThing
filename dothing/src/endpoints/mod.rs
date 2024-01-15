mod director;

use std::path::PathBuf;

use axum::routing::post;
use axum::Router;

use tower_http::services::ServeFile;

pub(crate) fn main_router() -> Router {
    Router::new()
        .route("/addition", post(director::recieve_addition_order))
        .route("/restart", post(director::recieve_restart_order))
        .route("/reconfig/http", post(""))
}

pub(crate) fn extras_router() -> Router {
    Router::new().route_service(
        "/favicon.ico",
        ServeFile::new(PathBuf::from("assets/favicon.ico")),
    )
}

use docker_api::{
    docker, image,
    opts::{
        ContainerConnectionOpts, ContainerCreateOpts, ContainerListOpts, ContainerRestartOpts,
        PullOpts,
    },
    Container, Docker,
};

async fn docker_shenanigans() {
    if let Ok(dock) = Docker::new("unix:///run/docker.sock") {
        let _info = dock.info().await.unwrap();

        // let images = dock.images();
        // let pullOpts = PullOpts::builder().image()

        let opts = ContainerListOpts::builder().all(true).build();

        let containers = dock.containers();

        let uuid = format!("DEVICE_UUID={}", uuid::Uuid::new_v4());

        let env_vars = vec!["RUST_LOG=debug", "ip=mqtt_broker", &uuid];
        let cmd = vec!["topic:co2"];

        let create_opts = ContainerCreateOpts::builder()
            .image("mrdahaniel/mocker")
            .name("MokerTest")
            .env(env_vars)
            .command(cmd)
            .build();

        let res = containers.create(&create_opts).await.unwrap();
        let inspect_res = res.inspect().await.unwrap();

        println!("{}", serde_json::to_string_pretty(&inspect_res).unwrap());

        let container_id = res.id();

        let network_opts = ContainerConnectionOpts::builder(container_id).build();

        dock.networks()
            .get("smart_campus_production_smart_uis")
            .connect(&network_opts)
            .await
            .unwrap();

        dock.containers()
            .get(container_id.clone())
            .start()
            .await
            .unwrap();

        println!("{:?}", containers.list(&opts).await.unwrap());
    };
}
