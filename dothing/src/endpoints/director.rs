use std::os::unix::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};

use docker_api::Docker;

use starduck::AdditionOrder;

use crate::dckr::ContainerBuilder;

pub async fn recieve_addition(
    Extension(docker): Extension<Docker>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(addition): Json<AdditionOrder>,
) -> Response {
    let container_opts = addition.build_container();

    match docker.containers().create(&container_opts).await {
        Ok(container) => todo!(),
        Err(e) => todo!(),
    };
    return (StatusCode::OK).into_response();

    (StatusCode::OK).into_response()
}
