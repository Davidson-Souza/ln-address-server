#[macro_use]
extern crate serde;

mod api;
mod cli;
mod phoenixd;

use api::config::ServerConfig;
use clap::Parser;
use cli::Cli;
use phoenixd::PhoenixdClient;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    let ph_client = PhoenixdClient {
        client: Client::default(),
        password: cli.phoenixd_password,
        host: cli.phoenixd_address.unwrap_or("127.0.0.1:9740".into())
    };

    let host = cli.api_host.unwrap_or("127.0.0.1".into());
    let port = cli.api_port.unwrap_or(8080);

    let config = ServerConfig {
        ph_client,
        users_dir: cli.users_dir.unwrap_or("./users".to_owned()),
        host: format!("{host}:{port}"),
    };

    api::api::run_server(config).await
}
