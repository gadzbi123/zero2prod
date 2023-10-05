use std::ops::Deref;

use actix_web::{web, HttpResponse, HttpResponseBuilder};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::types::uuid::Uuid;
use sqlx::PgPool;
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query!(
        r#"INSERT INTO subscriptions (id,name,email,subscribed_at) VALUES ($1,$2,$3,$4)"#,
        Uuid::new_v4(),
        form.name,
        form.email,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
pub async fn unsubscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    match sqlx::query!(
        r#"DELETE FROM subscriptions where name = $1 AND email = $2"#,
        form.name,
        form.email
    )
    .execute(db_pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to delete subscriber: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
