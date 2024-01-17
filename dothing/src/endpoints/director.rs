use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};

use docker_api::models::ContainerInspect200Response;
use docker_api::opts::ContainerRestartOpts;
use docker_api::Error::Fault;
use docker_api::{Container, Docker};

use reqwest::Client;
use serde_json::json;
use starduck::{AdditionOrder, ReconfigureOrder, ReconfigureType, RestartOrder};

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

    info!("Building container...");
    let container_opts = addition.build_container();

    info!("Using {:?} as creation options", &container_opts);

    let uuid = match addition.get_uuid() {
        Ok(uuid) => uuid,
        Err(e) => {
            error!("{e}");
            return (StatusCode::BAD_REQUEST, Json(e.to_string())).into_response();
        }
    };

    info!("Sending create signal to docker socket");

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

    let cont_name = &container.inspect().await.unwrap().name.unwrap();

    info!("Container created with name: {}", cont_name);

    info!("Building network connection options...");

    let network_opts = addition.build_connection(container.id());

    info!("Built network connection options");
    info!("Connecting container to network {}", &addition.network_name);

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

    info!("Connected container to network {}", &addition.network_name);

    info!("Starting container {}", cont_name);

    if let Err(e) = container.start().await {
        error!("Could not start container {}", container.id());

        if let Fault { code, message } = e {
            let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
            return (status_code, message).into_response();
        }

        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    info!("Started container {}", cont_name);

    info!("Process successful!");

    return (StatusCode::OK).into_response();
}

pub async fn recieve_restart_order(
    method: Method,
    Extension(docker): Extension<Docker>,
    Extension(mapping): Extension<ContMap>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(restart): Json<RestartOrder>,
) -> Response {
    async fn restart_container(container: &Container) -> Response {
        info!("Building RestartOptions");

        let restart_opts = ContainerRestartOpts::builder().build();

        info!("Using RestartOptions: {:?}", &restart_opts);

        let cont_name = &container.inspect().await.unwrap().name.unwrap();

        info!("Restarting container {}...", cont_name);

        if let Err(e) = container.restart(&restart_opts).await {
            if let Fault { code, message } = e {
                let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
                return (status_code, message).into_response();
            }

            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }

        info!("Container {} restarted", cont_name);

        return (StatusCode::OK).into_response();
    }

    async fn start_stopped_container(container: &Container) -> Response {
        let cont_name = &container.inspect().await.unwrap().name.unwrap();

        info!("Starting stopped container {}", cont_name);

        if let Err(e) = container.start().await {
            if let Fault { code, message } = e {
                let status_code = StatusCode::from_u16(code.as_u16()).unwrap();
                return (status_code, message).into_response();
            }

            warn!("Failed to start stopped container {}", cont_name);

            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }

        info!("Started stopped container {}", cont_name);

        return (StatusCode::OK).into_response();
    }

    info!("{method} request from {addr}");

    let uuid = restart.uuid.clone().unwrap();
    let mapp = mapping.lock().unwrap().get(&uuid).cloned();

    info!("Looking for container with uuid {} in the mapping...", uuid);

    if let Some(container_id) = mapp {
        let cont = docker.containers().get(container_id.clone());
        let cont_info = cont.inspect().await.unwrap();

        match cont_info.state.and_then(|s| s.status) {
            Some(status) if status.as_str() != RUNNING => {
                return start_stopped_container(&cont).await
            }
            Some(_) => return restart_container(&cont).await,
            None => {
                return (
                    StatusCode::NO_CONTENT,
                    Json(json!({"msg": "Container didn't have a status"})),
                )
                    .into_response()
            }
        }
    };

    warn!("Could not find container with uuid {} in the mapping", uuid);

    info!("Searching for container in the network...");

    if let Some(cont) = restart.find_container(&docker).await {
        info!("Container found");
        info!("Adding it to the register");

        mapping.lock().unwrap().insert(uuid, cont.id().clone());

        info!("Container added to the register");

        let cont_info = cont.inspect().await.unwrap();

        match cont_info.state.and_then(|s| s.status) {
            Some(status) if status.as_str() == RUNNING => return restart_container(&cont).await,
            Some(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"msg": "Container can't be restarted"})),
                )
                    .into_response()
            }
            None => {
                return (
                    StatusCode::NO_CONTENT,
                    Json(json!({"msg": "Container didn't have a status"})),
                )
                    .into_response()
            }
        }
    };

    warn!("Could not find container");

    let json = json!({ "msg": "Couldn't find container" });
    (StatusCode::NOT_FOUND, Json(json)).into_response()
}

pub async fn recieve_update_order(
    method: Method,
    Extension(docker): Extension<Docker>,
    Extension(mapping): Extension<ContMap>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(update): Json<ReconfigureOrder>,
) -> Response {
    async fn send_reconfigure_request(
        cont: Container,
        reconfig_order: &ReconfigureOrder,
    ) -> Response {
        match (cont.inspect().await, &reconfig_order.reconfig) {
            (
                Ok(cont_info),
                starduck::ReconfigureType::Http {
                    endpoint,
                    method,
                    payload,
                    port,
                },
            ) => {
                info!("Building reqwest client");

                let cli = Client::new();
                let domain = cont_info.name.unwrap();
                let url = format!("http://{}:{}{}", domain, port, endpoint.to_string_lossy());

                info!("Built reqwest client");

                info!("Sending request to {}", &url);

                let response = match method.clone() {
                    Method::PUT => cli.put(&url).json(payload).send().await,
                    _ => {
                        let json = json!({ "msg": "Could not build request" });
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response();
                    }
                };

                info!("Request made to {}", &url);

                info!("Got response from {}", &url);

                match response {
                    Ok(response) => {
                        let code = response.status();
                        return (StatusCode::from_u16(code.as_u16()).unwrap()).into_response();
                    }
                    Err(e) => {
                        return (StatusCode::from_u16(e.status().unwrap().as_u16())
                            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
                        .into_response();
                    }
                };
            }
            (Err(e), _) => match e {
                Fault { code, message } => {
                    info!("Unexpected error: {}", &message);

                    let json = json!({"msg": message});
                    return (StatusCode::from_u16(code.as_u16()).unwrap(), Json(json))
                        .into_response();
                }
                _ => return (StatusCode::IM_A_TEAPOT).into_response(),
            },
        }
    }

    info!("{method} request from {addr}");

    let uuid = update.uuid.clone().unwrap();
    let mapp = mapping.lock().unwrap().get(&uuid).cloned();

    info!("Looking for container with uuid {} in the mapping...", uuid);

    if let Some(container_id) = mapp {
        let cont = docker.containers().get(container_id.clone());

        return send_reconfigure_request(cont, &update).await;
    };

    warn!("Could not find container with uuid {} in the mapping", uuid);

    info!("Searching for container in the network...");

    if let Some(cont) = update.find_container(&docker).await {
        info!("Container found");
        info!("Adding it to the register");

        mapping.lock().unwrap().insert(uuid, cont.id().clone());

        info!("Container added to the register");

        return send_reconfigure_request(cont, &update).await;
    };

    warn!("Could not find container");

    let json = json!({ "msg": "Could not find container!" });
    (StatusCode::NOT_FOUND, Json(json)).into_response()
}
