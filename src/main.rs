// use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::{
    configuration::{self, get_configuration},
    startup::run,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}