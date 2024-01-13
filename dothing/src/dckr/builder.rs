use anyhow::{Context, Result};
use docker_api::Docker;
use http::Uri;

const WIN_SOCKET_URI: &str = "tcp://127.0.0.1:2376";
const UNIX_SOCKET_URI: &str = "unix:///run/docker.sock";

pub fn build_docker() -> Docker {
    let uri = match () {
        _ if cfg!(windows) => WIN_SOCKET_URI,
        _ if cfg!(unix) => UNIX_SOCKET_URI,
        _ => {
            error!("Could not stablish platform");
            std::process::exit(-1)
        }
    };

    if let Ok(dckr) = Docker::new(uri) {
        return dckr;
    }

    error!("Could not connect to docker.socket. Is docker running?");
    quit::with_code(1);
}
