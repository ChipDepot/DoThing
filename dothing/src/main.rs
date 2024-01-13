mod actions;
mod dckr;
mod endpoints;
mod res;

#[macro_use]
extern crate log;

use std::net::SocketAddr;

use docker_api::Docker;

use axum::{Extension, Router};
use tokio::net::TcpListener;

use starduck::utils::PORT;

const DEFAULT_PORT: u16 = 8050;

#[tokio::main]
async fn main() {
    env_logger::init();

    let docker = dckr::build_docker();

    let port = starduck::utils::get(PORT).unwrap_or(DEFAULT_PORT);

    let app = Router::new()
        .nest("/apps", endpoints::main_router())
        .layer(Extension(docker));

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
