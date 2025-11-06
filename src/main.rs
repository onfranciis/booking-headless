mod middlewares;
mod routes;
mod structs;
mod utils;

use crate::{
    routes::{
        appointment_routes, auth_routes, service_routes, user_routes,
        utils_routes::{home, route_not_found},
    },
    utils::response_utils::{json_error_handler, path_error_handler},
};
use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL was not set in .env file");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool.");

    println!("Running database migrations...");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations.");

    println!("Migrations complete.");

    println!("🚀 Server starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        let path_config = web::PathConfig::default().error_handler(path_error_handler);
        let json_config = web::JsonConfig::default().error_handler(json_error_handler);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(path_config)
            .app_data(json_config)
            .configure(auth_routes::auth_config)
            .configure(user_routes::user_config)
            .configure(service_routes::service_config)
            .configure(appointment_routes::appointment_config)
            .service(home)
            .default_service(web::to(route_not_found))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
