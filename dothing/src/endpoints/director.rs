use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};

use docker_api::Docker;
use docker_api::Error::Fault;

use docker_api::opts::ContainerRestartOpts;
use reqwest::Client;
use serde_json::json;
use starduck::{AdditionOrder, ReconfigureOrder, RestartOrder};

use crate::ContMap;

use crate::dckr::{ConnectionBuilder, ContainerBuilder, FindContainer};

const RUNNING: &str = "running";

pub async fn recieve_addition_order(
    method: Method,
    Extension(docker): Extension<Docker>,
    Extension(mapping): Extension<ContMap>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(addition): Json<AdditionOrder>,
) -> Response {
    info!("{method} request from {addr}");

    let container_opts = addition.build_container();
    let uuid = match addition.get_uuid() {
        Ok(uuid) => uuid,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response(),
    };

    let container = match docker.containers().create(&container_opts).await {
        Ok(container) => {
            mapping.lock().unwrap().insert(uuid, container.id().clone());
            container
        }
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
    Extension(mapping): Extension<ContMap>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(restart): Json<RestartOrder>,
) -> Response {
    info!("{method} request from {addr}");

    if let Some(cont) = restart.find_container(&docker).await {
        let cont_info = cont.inspect().await.unwrap();

        match cont_info.state.and_then(|s| s.status) {
            Some(status) if status.as_str() == RUNNING => {
                let restart_opts = ContainerRestartOpts::builder().build();

                if let Err(e) = cont.restart(&restart_opts).await {
                    if let Fault { code, message } = e {
                        let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
                        return (status_code, message).into_response();
                    }

                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }

                return (StatusCode::OK).into_response();
            }
            Some(_) => {
                if let Err(e) = cont.start().await {
                    if let Fault { code, message } = e {
                        let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
                        return (status_code, message).into_response();
                    }

                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }

                return (StatusCode::OK).into_response();
            }
            None => todo!(),
        }
    };

    let json = json!({
        "msg": "Could not find container"
    });

    (StatusCode::NOT_FOUND, Json(json)).into_response()
}

pub async fn recieve_update_order(
    method: Method,
    Extension(docker): Extension<Docker>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(update): Json<ReconfigureOrder>,
) -> Response {
    info!("{method} request from {addr}");

    if let Some(cont) = update.find_container(&docker).await {
        match (cont.inspect().await, &update.reconfig) {
            (
                Ok(cont_info),
                starduck::ReconfigureType::Http {
                    endpoint,
                    method,
                    payload,
                    port,
                },
            ) => {
                let cli = Client::new();
                let domain = cont_info.name.unwrap();
                let url = format!("http://{}:{}{}", domain, port, endpoint.to_string_lossy());

                let response = match method.clone() {
                    Method::PUT => cli.put(url).json(payload).send().await,
                    _ => {
                        let json = json!({ "msg": "Could not build request" });
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response();
                    }
                };

                match response {
                    Ok(response) => {
                        let code = response.status();
                        return (StatusCode::from_u16(code.as_u16()).unwrap()).into_response();
                    }
                    Err(e) => {
                        return (StatusCode::from_u16(e.status().unwrap().as_u16())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
                        .into_response()
                    }
                };
            }
            (Err(e), _) => match e {
                Fault { code, message } => {
                    let json = json!({"msg": message});

                    return (StatusCode::from_u16(code.as_u16()).unwrap(), Json(json))
                        .into_response();
                }
                _ => return (StatusCode::IM_A_TEAPOT).into_response(),
            },
        }
    };

    let json = json!({ "msg": "Could not find container!" });
    (StatusCode::NOT_FOUND, Json(json)).into_response()
}
