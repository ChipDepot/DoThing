use docker_api::{
    opts::{ContainerConnectionOpts, ContainerCreateOpts, ContainerListOpts},
    Container, Docker,
};

use starduck::Doctrine;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    if let Ok(docker) = Docker::new("unix:///run/docker.sock") {
        let _info = docker.info().await.unwrap();

        let opts = ContainerListOpts::builder().all(true).build();

        let containers = docker.containers();

        let env_vars = vec!["RUST_LOG=mocker", "ip=mqtt_broker"];
        let cmd = vec!["topic:co2"];

        let create_opts = ContainerCreateOpts::builder()
            .image("mrdahaniel/mocker")
            .name("MokerTest")
            .env(env_vars)
            .command(cmd)
            .build();

        let res = containers.create(&create_opts).await.unwrap();
        let inspect_res = res.inspect().await.unwrap();

        res.start().await.unwrap();

        println!("{}", serde_json::to_string_pretty(&inspect_res).unwrap());

        let container_id = res.id();

        let network_opts = ContainerConnectionOpts::builder(container_id).build();

        if let Err(e) = docker
            .networks()
            .get("smart_campus_production_smart_uis")
            .connect(&network_opts)
            .await
        {
            println!("Err: {}", e);
        };

        docker
            .containers()
            .get(container_id.clone())
            .start()
            .await
            .unwrap();
        // println!("{:?}", containers.list(&opts).await.unwrap());
    };
}
