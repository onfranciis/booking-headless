use crate::{
    routes::utils_routes::internal_server_error_response,
    structs::{
        db_struct::{CreateService, Service, UpdateService},
        response_struct::ApiResponse,
    },
};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use uuid::Uuid;

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn create_service(pool: web::Data<PgPool>, body: web::Json<CreateService>) -> impl Responder {
    let new_service = body.into_inner();

    match sqlx::query_as!(
        Service,
        r#"
        INSERT INTO services (
            user_id, service_name, description, price, 
            duration_minutes, category
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        new_service.user_id,
        new_service.service_name,
        new_service.description,
        new_service.price,
        new_service.duration_minutes,
        new_service.category
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(service) => HttpResponse::Created().json(ApiResponse {
            success: true,
            data: Some(service),
            message: Some("Service created successfully".to_string()),
        }),

        Err(e) => {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_foreign_key_violation() {
                    return HttpResponse::NotFound().json(ApiResponse::<()> {
                        success: false,
                        data: None,
                        message: Some("The user_id provided does not exist.".to_string()),
                    });
                }
            }

            internal_server_error_response(e.to_string())
        }
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

async fn get_service_by_id(path: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let service_id = path.into_inner();

    match sqlx::query_as!(
        Service,
        r#"
        SELECT *
        FROM services 
        WHERE id = $1
        "#,
        service_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(service) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(service),
            message: Some("Service retrieved successfully".to_string()),
        }),

        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Service not found".to_string()),
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

async fn get_all_services(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query_as!(
        Service,
        r#"
        SELECT *
        FROM services
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(services) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(services),
            message: Some("Services retrieved successfully".to_string()),
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

async fn update_service(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateService>,
) -> impl Responder {
    let service_id = path.into_inner();
    let fields_to_update = body.into_inner();

    match sqlx::query_as!(
        Service,
        r#"
        UPDATE services SET
            service_name = COALESCE($1, service_name),
            description = COALESCE($2, description),
            price = COALESCE($3, price),
            duration_minutes = COALESCE($4, duration_minutes),
            category = COALESCE($5, category),
            updated_at = NOW()
        WHERE id = $6
        RETURNING *
        "#,
        fields_to_update.service_name,
        fields_to_update.description,
        fields_to_update.price,
        fields_to_update.duration_minutes,
        fields_to_update.category,
        service_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(service) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(service),
            message: Some("Service updated successfully".to_string()),
        }),

        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Service not found".to_string()),
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

async fn delete_service(path: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let service_id = path.into_inner();

    match sqlx::query_as!(
        Service,
        r#"
        DELETE FROM services
        WHERE id = $1
        RETURNING *
        "#,
        service_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(deleted_service) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(deleted_service),
            message: Some("Service deleted successfully".to_string()),
        }),

        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Service not found".to_string()),
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

pub fn service_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/services")
            .route("", web::post().to(create_service))
            .route("", web::get().to(get_all_services))
            .route("/{id}", web::get().to(get_service_by_id))
            .route("/{id}", web::patch().to(update_service))
            .route("/{id}", web::delete().to(delete_service)),
    );
}
