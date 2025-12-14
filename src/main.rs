mod config;
mod middlewares;
mod routes;
mod structs;
mod utils;

use crate::{
    config::Config,
    routes::{
        appointment_routes, auth_routes, service_routes, user_routes,
        utils_routes::{home, route_not_found},
    },
    utils::response_utils::{json_error_handler, path_error_handler, query_error_handler},
};
use actix_web::{App, HttpServer, web};
use deadpool_redis::{Config as RedisConfig, Runtime};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    let bind_address = format!("127.0.0.1:{}", config.port);
    let http_client = reqwest::Client::new();
    let redis_cfg = RedisConfig::from_url(&config.redis_url);
    let redis_pool = redis_cfg.create_pool(Some(Runtime::Tokio1)).unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool.");

    println!("Running database migrations...");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations.");

    println!("Migrations complete.");

    println!("🚀 Server starting at http://{}", bind_address);

    HttpServer::new(move || {
        let path_config = web::PathConfig::default().error_handler(path_error_handler);
        let json_config = web::JsonConfig::default().error_handler(json_error_handler);
        let query_config = web::QueryConfig::default().error_handler(query_error_handler);

        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(http_client.clone()))
            .app_data(path_config)
            .app_data(json_config)
            .app_data(query_config)
            .configure(auth_routes::auth_config)
            .configure(user_routes::user_config)
            .configure(service_routes::service_config)
            .configure(appointment_routes::appointment_config)
            .service(home)
            .default_service(web::to(route_not_found))
    })
    .bind(bind_address)?
    .run()
    .await
}
