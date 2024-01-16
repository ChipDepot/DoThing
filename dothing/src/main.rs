mod actions;
mod dckr;
mod endpoints;
mod res;

#[macro_use]
extern crate log;

use std::net::SocketAddr;
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

use axum::{Extension, Router};
use docker_api::Id;
use tokio::net::TcpListener;

use starduck::utils::PORT;
use uuid::Uuid;

const DEFAULT_PORT: u16 = 8050;

type ContMap = Arc<Mutex<HashMap<Uuid, Id>>>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let docker = dckr::build_docker();
    let container_map = Arc::new(Mutex::new(HashMap::<Uuid, Id>::new()));

    let app = Router::new()
        .nest("/", endpoints::main_router())
        .nest_service("/", endpoints::extras_router())
        .layer(Extension(docker))
        .layer(Extension(container_map));

    let port = starduck::utils::get(PORT).unwrap_or(DEFAULT_PORT);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let tcp_listener = TcpListener::bind(addr).await.unwrap_or_else(|e| {
        error!("Could not start server: {e}");
        std::process::exit(-1);
    });

    info!("Initializing server at {}", &addr);

    axum::serve(
        tcp_listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap_or_else(|e| {
        error!("Could not start server: {e}");
        std::process::exit(-1);
    });
}
