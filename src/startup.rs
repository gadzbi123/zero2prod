use crate::routes::{check_health, greet, subscribe, unsubscribe};
use actix_web::{
    dev::Server, get, post, web, App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
    Responder,
};
use sqlx::{PgConnection, PgPool};
use std::net::TcpListener;
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(greet))
            .route("/check_health", web::get().to(check_health))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/unsubscriptions", web::post().to(unsubscribe))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
