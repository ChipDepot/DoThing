mod director;

use std::path::PathBuf;

use axum::routing::post;
use axum::Router;

use tower_http::services::ServeFile;

pub(crate) fn main_router() -> Router {
    Router::new()
        .route("/addition", post(director::recieve_addition_order))
        .route("/restart", post(director::recieve_restart_order))
        .route("/reconfig/http", post(director::recieve_update_order))
}

pub(crate) fn extras_router() -> Router {
    Router::new().route_service(
        "/favicon.ico",
        ServeFile::new(PathBuf::from("assets/favicon.ico")),
    )
}
