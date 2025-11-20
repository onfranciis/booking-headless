use std::time::Duration;

use crate::{
    config::Config,
    middlewares::auth_middleware::AuthenticatedUser,
    routes::utils_routes::{internal_server_error_response, not_found_response},
    structs::{
        db_struct::{CreateService, Service, UpdateService},
        response_struct::ApiResponse,
        util_struct::UploadResponse,
    },
    utils::auth_utils::get_gcs_client,
};
use actix_web::{HttpResponse, Responder, web};
use gcloud_storage::sign::{SignedURLMethod, SignedURLOptions};
use sqlx::PgPool;
use uuid::Uuid;

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn create_service(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    body: web::Json<CreateService>,
) -> impl Responder {
    let mut new_service = body.into_inner();
    let user_id = user.user_id;

    // Check if service_name is empty or only whitespace
    if new_service.service_name.trim().is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("Service name cannot be empty.".to_string()),
        });
    } else {
        new_service.service_name = new_service.service_name.trim().to_string();
    }

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
        user_id,
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
                    return not_found_response("The user_id provided does not exist.".to_string());
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
        Ok(service) => {
            let user_is_active = sqlx::query_scalar!(
                r#"SELECT is_active as "is_active!: bool" FROM users WHERE id = $1"#,
                service.user_id
            )
            .fetch_optional(pool.get_ref())
            .await;

            match user_is_active {
                Ok(Some(true)) => HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(service),
                    message: Some("Service retrieved successfully".to_string()),
                }),

                Ok(Some(false)) => {
                    let response = ApiResponse::<()> {
                        success: false,
                        data: None,
                        message: Some("The owner of this service is not active".to_string()),
                    };

                    return HttpResponse::Forbidden().json(response);
                }
                _ => {
                    return not_found_response("User not found.".to_string());
                }
            }
        }

        Err(sqlx::Error::RowNotFound) => not_found_response("Service not found".to_string()),

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
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    body: web::Json<UpdateService>,
) -> impl Responder {
    let service_id = path.into_inner();
    let user_id = user.user_id;
    let mut fields_to_update = body.into_inner();

    // Check if service_name is empty or only whitespace
    if let Some(service_name) = fields_to_update.service_name.clone() {
        if service_name.trim().is_empty() {
            fields_to_update.service_name = None;
        } else {
            fields_to_update.service_name = Some(service_name.trim().to_string());
        }
    }

    let service_to_update =
        match sqlx::query_as!(Service, "SELECT * FROM services WHERE id = $1", service_id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(service) => service,

            Err(sqlx::Error::RowNotFound) => {
                return not_found_response("Service not found".to_string());
            }

            Err(e) => {
                return internal_server_error_response(e.to_string());
            }
        };

    // Check ownership
    if service_to_update.user_id != user_id {
        return HttpResponse::Forbidden().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("You do not have permission to edit this service.".to_string()),
        });
    }

    // The user is authorized, now we can update the service
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

async fn delete_service(
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let service_id = path.into_inner();
    let user_id = user.user_id;

    let service_to_delete =
        match sqlx::query_as!(Service, "SELECT * FROM services WHERE id = $1", service_id)
            .fetch_one(pool.get_ref())
            .await
        {
            Ok(service) => service,

            Err(sqlx::Error::RowNotFound) => {
                return not_found_response("Service not found".to_string());
            }

            Err(e) => {
                return internal_server_error_response(e.to_string());
            }
        };

    // Check ownership
    if service_to_delete.user_id != user_id {
        return HttpResponse::Forbidden().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("You do not have permission to delete this service.".to_string()),
        });
    }

    // The user is authorized, now we can delete the service
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

async fn get_service_upload_url(
    path: web::Path<Uuid>,
    user: AuthenticatedUser,
    config: web::Data<Config>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let service_id = path.into_inner();
    let user_id = user.user_id;

    // Check ownership
    let service = match sqlx::query_as!(Service, "SELECT * FROM services WHERE id = $1", service_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(service) => service,

        Err(sqlx::Error::RowNotFound) => {
            return not_found_response("Service not found".to_string());
        }

        Err(e) => return internal_server_error_response(e.to_string()),
    };

    if service.user_id != user_id {
        return HttpResponse::Forbidden().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: Some("You do not have permission to edit this service.".to_string()),
        });
    }

    // Generate URLs
    let file_path = format!("services/{}/image.jpg", service_id);
    let bucket_name = &config.gcs_bucket_name;
    let client = get_gcs_client(&config).await;

    let options = SignedURLOptions {
        expires: Duration::from_secs(60 * 7), // 7 minutes
        method: SignedURLMethod::PUT,
        ..Default::default()
    };

    let signed_url = match client
        .signed_url(bucket_name, &file_path, None, None, options)
        .await
    {
        Ok(url) => url,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    let public_url = format!(
        "https://storage.googleapis.com/{}/{}",
        bucket_name, file_path
    );

    // Update the service record
    if let Err(e) = sqlx::query!(
        "UPDATE services SET image_url = $1 WHERE id = $2",
        public_url,
        service_id
    )
    .execute(pool.get_ref())
    .await
    {
        return internal_server_error_response(e.to_string());
    }

    let response = ApiResponse {
        success: true,
        data: Some(UploadResponse {
            signed_upload_url: signed_url,
            public_url: public_url,
        }),
        message: Some("Upload URL generated.".to_string()),
    };

    HttpResponse::Ok().json(response)
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
            .route("/{id}/upload-url", web::get().to(get_service_upload_url))
            .route("/{id}", web::get().to(get_service_by_id))
            .route("/{id}", web::patch().to(update_service))
            .route("/{id}", web::delete().to(delete_service)),
    );
}
