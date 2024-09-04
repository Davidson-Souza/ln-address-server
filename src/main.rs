#[macro_use]
extern crate serde;

mod api;
mod cli;
mod nostr;
mod phoenixd;

use api::config::ServerConfig;
use clap::Parser;
use cli::Cli;
use hex_conservative::DisplayHex;
use nostr::zap_handler::ZapHandler;
use phoenixd::PhoenixdClient;
use reqwest::Client;
use secp256k1::Secp256k1;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    let ph_client = PhoenixdClient {
        client: Client::default(),
        password: cli.phoenixd_password,
        host: cli.phoenixd_address.unwrap_or("127.0.0.1:9740".into()),
    };

    let host = cli.api_host.unwrap_or("127.0.0.1".into());
    let port = cli.api_port.unwrap_or(8080);
    // show this back as we need it for the lnurl json
    let pubkey = cli
        .secret_key
        .x_only_public_key(&Secp256k1::new())
        .0
        .serialize()
        .to_lower_hex_string();

    let (zap_handler, sender) = ZapHandler::new(ph_client.clone(), cli.secret_key).await;

    let _handler = tokio::task::spawn(zap_handler.run());
    let config = ServerConfig {
        ph_client,
        users_dir: cli.users_dir.unwrap_or("./users".to_owned()),
        host: format!("{host}:{port}"),
        zap_sender: sender,
        zap_pk: pubkey,
    };

    api::api::run_server(config).await
}
