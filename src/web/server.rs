use crate::{
    env::{HTTP_PORT, HTTP_SERVER},
    web::routes,
};
use actix_web::{middleware, App, HttpServer};
use anyhow::anyhow;
use log;
use std::env;

#[actix_web::main]
pub async fn main() -> anyhow::Result<()> {
    let http_server = env::var(HTTP_SERVER).unwrap();
    let http_port = match env::var(HTTP_PORT).unwrap().parse::<u16>() {
        Ok(p) => p,
        Err(err) => return Err(anyhow!(err)),
    };

    log::info!("starting HTTP server at {http_server}:{http_port}");

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(routes::greet)
    })
    .bind((http_server, http_port))?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
