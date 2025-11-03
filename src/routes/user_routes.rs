use std::collections::HashMap;

use crate::{
    routes::utils_routes::{conflict_reponse, internal_server_error_response, not_found_response},
    structs::{
        db_struct::{Appointment, CreateUser, Service, UpdateUser, User, UserWithServices},
        response_struct::ApiResponse,
    },
};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use uuid::Uuid;

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn create_user(pool: web::Data<PgPool>, body: web::Json<CreateUser>) -> impl Responder {
    let new_user = body.into_inner();

    match sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            username, business_name, email, location, phone_number, 
            description, phone_number_is_whatsapp
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        new_user.username,
        new_user.business_name,
        new_user.email,
        new_user.location,
        new_user.phone_number,
        new_user.description,
        new_user.phone_number_is_whatsapp.unwrap_or(false)
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(created_user) => HttpResponse::Created().json(ApiResponse {
            message: Some("User created successfully".to_string()),
            data: Some(created_user),
            success: true,
        }),

        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                conflict_reponse("Username or email already exists".to_string())
            } else {
                internal_server_error_response(db_err.to_string())
            }
        }

        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn get_user_by_id(path: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = path.into_inner();

    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(user) => HttpResponse::Ok().json(ApiResponse {
            message: Some("User retrieved successfully".to_string()),
            data: Some(user),
            success: true,
        }),

        Err(sqlx::Error::RowNotFound) => not_found_response("User not found".to_string()),

        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn get_all_users(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(users) => HttpResponse::Ok().json(ApiResponse {
            message: Some("Users retrieved successfully".to_string()),
            data: Some(users),
            success: true,
        }),

        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn get_all_users_with_services(pool: web::Data<PgPool>) -> impl Responder {
    // Fetch all users
    let users_result = sqlx::query_as!(User, r#"SELECT * FROM users"#)
        .fetch_all(pool.get_ref())
        .await;

    let all_users = match users_result {
        Ok(users) => users,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    // Fetch all services
    let services_result = sqlx::query_as!(
        Service,
        r#"
        SELECT * FROM services
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    let all_services = match services_result {
        Ok(services) => services,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    // Combine them
    let mut services_map: HashMap<Uuid, Vec<Service>> = HashMap::new();

    for service in all_services {
        services_map
            .entry(service.user_id)
            .or_default()
            .push(service);
    }

    let response: Vec<UserWithServices> = all_users
        .into_iter()
        .map(|user| {
            let services = services_map.remove(&user.id).unwrap_or_else(Vec::new);
            UserWithServices { user, services }
        })
        .collect();

    HttpResponse::Ok().json(ApiResponse {
        message: Some("Users with services retrieved successfully".to_string()),
        data: Some(response),
        success: true,
    })
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn update_user(
    path: web::Path<Uuid>,
    updated_user: web::Json<UpdateUser>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = path.into_inner();
    let updated_user = updated_user.into_inner();

    match sqlx::query_as!(
        User,
        r#"
        UPDATE users SET
            username = COALESCE($1, username),
            business_name = COALESCE($2, business_name),
            email = COALESCE($3, email),
            location = COALESCE($4, location),
            phone_number = COALESCE($5, phone_number),
            description = COALESCE($6, description),
            phone_number_is_whatsapp = COALESCE($7, phone_number_is_whatsapp),
            updated_at = NOW()
        WHERE id = $8
        RETURNING *
        "#,
        updated_user.username,
        updated_user.business_name,
        updated_user.email,
        updated_user.location,
        updated_user.phone_number,
        updated_user.description,
        updated_user.phone_number_is_whatsapp,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(user) => HttpResponse::Ok().json(ApiResponse {
            message: Some("User updated successfully".to_string()),
            data: Some(user),
            success: true,
        }),

        Err(sqlx::Error::RowNotFound) => not_found_response("User not found".to_string()),

        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                conflict_reponse("Username or email already exists".to_string())
            } else {
                internal_server_error_response(db_err.to_string())
            }
        }

        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn get_appointments_for_user(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = path.into_inner();

    match sqlx::query_as!(
        Appointment,
        r#"
        SELECT * FROM appointments 
        WHERE business_id = $1
        ORDER BY appointment_time DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(appointments) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(appointments),
            message: Some("Appointments retrieved successfully".to_string()),
        }),

        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/with-services", web::get().to(get_all_users_with_services))
            .route(
                "/{id}/appointments",
                web::get().to(get_appointments_for_user),
            )
            .route("/{id}", web::get().to(get_user_by_id))
            .route("/{id}", web::patch().to(update_user))
            .route("", web::get().to(get_all_users))
            .route("", web::post().to(create_user)),
    );
}
