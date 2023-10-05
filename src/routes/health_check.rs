use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer, Responder};

pub async fn greet(req: HttpRequest) -> String {
    let x = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}", x)
}
pub async fn check_health() -> HttpResponseBuilder {
    HttpResponse::Ok()
}
