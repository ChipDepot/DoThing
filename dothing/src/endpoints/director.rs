use std::net::SocketAddr;
use std::path::PathBuf;

use axum::extract::ConnectInfo;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};

use docker_api::Docker;
use docker_api::Error::Fault;

use docker_api::opts::ContainerRestartOpts;
use serde_json::json;
use starduck::{AdditionOrder, RestartOrder};

use crate::dckr::{ConnectionBuilder, ContainerBuilder, FindContainer};

pub async fn recieve_addition_order(
    method: Method,
    Extension(docker): Extension<Docker>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(addition): Json<AdditionOrder>,
) -> Response {
    info!("{method} request from {addr}");

    let container_opts = addition.build_container();

    let container = match docker.containers().create(&container_opts).await {
        Ok(container) => container,
        Err(e) => {
            error!("Error while creating container: {e}");
            if let Fault { code, message } = e {
                let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
                return (status_code, message).into_response();
            }

            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    let network_opts = addition.build_connection(container.id());

    if let Err(e) = docker
        .networks()
        .get(&addition.network_name)
        .connect(&network_opts)
        .await
    {
        error!(
            "Could not connect container {} to {}",
            container.id(),
            &addition.network_name
        );

        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    if let Err(e) = container.start().await {
        error!("Could not start container {}", container.id());

        if let Fault { code, message } = e {
            let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
            return (status_code, message).into_response();
        }

        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    return (StatusCode::OK).into_response();
}

pub async fn recieve_restart_order(
    method: Method,
    Extension(docker): Extension<Docker>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(restart): Json<RestartOrder>,
) -> Response {
    info!("{method} request from {addr}");

    if let Some(cont) = restart.find_container(&docker).await {
        let restart_opts = ContainerRestartOpts::builder().build();

        if let Err(e) = cont.restart(&restart_opts).await {
            if let Fault { code, message } = e {
                let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
                return (status_code, message).into_response();
            }

            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }

        return (StatusCode::OK).into_response();
    };

    let json = json!({
        "msg": "Could not find container"
    });

    (StatusCode::NOT_FOUND, Json(json)).into_response()
}
