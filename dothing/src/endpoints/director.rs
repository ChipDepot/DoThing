use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};

use docker_api::Docker;
use starduck::Application;
use starduck::ExecMessage;

pub async fn recieve_directive(
    Extension(docker): Extension<Docker>,
    Json(message): Json<ExecMessage>,
) -> Response {
    (StatusCode::OK).into_response()
}
