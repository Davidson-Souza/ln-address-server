use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;

use super::callback::ln_url_callback;
use super::config::ServerConfig;
use super::lnaddress::well_known;

/// Actually runs the server
pub async fn run_server(config: ServerConfig) -> std::io::Result<()> {
    let host = config.host.clone();
    let _ = HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .service(ln_url_callback)
            .service(well_known)
            .app_data(Data::new(config.clone()))
    })
    .bind(host)?
    .run()
    .await;

    Ok(())
}
