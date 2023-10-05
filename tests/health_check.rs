use sqlx::types::uuid::Uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup;
pub struct TestApp {
    address: String,
    db_pool: PgPool,
}
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;
    let server = startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}", port);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(sqlx::query(
            format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str(),
        ))
        .await
        .expect("Failed to create database");
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres/database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}
// pub async fn drop_database(db_pool: PgPool, config: DatabaseSettings) {
//     let drop_command = format!(r#"DROP DATABASE "{}";"#, config.database_name);
//     db_pool
//         .execute(sqlx::query(drop_command.as_str()))
//         .await
//         .expect(&drop_command);
// }

#[tokio::test]
async fn check_health_works() {
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/check_health", &address))
        .send()
        .await
        .expect("Failed to make request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_return_200_on_valid_data() {
    let TestApp { address, db_pool } = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");
    let client = reqwest::Client::new();

    let body = "name=kacper%20nitkiewicz&email=crg1234%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to make request");

    assert_eq!(
        response.status().as_u16(),
        200,
        "API did not respond correctly on valid body request"
    );

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.name, "kacper nitkiewicz");
    assert_eq!(saved.email, "crg1234@gmail.com");
}
// #[tokio::test]
// async fn unsubscribe_return_200_on_valid_data() {
//     let TestApp { address, db_pool } = spawn_app().await;
//     let configuration = get_configuration().expect("Failed to read configuration");
//     let connection_string = configuration.database.connection_string();
//     let client = reqwest::Client::new();

//     let body = "name=kacper%20nitkiewicz&email=crg1234%40gmail.com";
//     let response = client
//         .post(&format!("{}/unsubscriptions", address))
//         .header("Content-Type", "application/x-www-form-urlencoded")
//         .body(body)
//         .send()
//         .await
//         .expect("Failed to make request");

//     assert_eq!(
//         response.status().as_u16(),
//         200,
//         "API did not respond correctly on valid body request"
//     );

//     let deleted = sqlx::query!("SELECT email, name FROM subscriptions")
//         .fetch_one(&db_pool)
//         .await;
//     assert!(deleted.is_err());
// }
#[tokio::test]
async fn subscribe_return_400_on_invalid_data() {
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();
    let invalid_data = vec![
        ("name=kacper%20nitkiewicz", "missing email"),
        ("email=crg1234%40gmail.com", "missing name"),
        ("", "missing name and email"),
    ];
    for (invalid_body, error_message) in invalid_data {
        let result = client
            .post(&format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to make request");
        assert_eq!(
            400,
            result.status().as_u16(),
            "API did not failed with 400 Bad Request when payload was {}",
            error_message
        )
    }
}
