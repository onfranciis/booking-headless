use crate::{
    routes::utils_routes::{
        bad_request_response, internal_server_error_response, not_found_response,
    },
    structs::{
        db_struct::{Appointment, CreateAppointment},
        response_struct::ApiResponse,
    },
};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use uuid::Uuid;

async fn create_appointment(
    pool: web::Data<PgPool>,
    body: web::Json<CreateAppointment>,
) -> impl Responder {
    let new_appt = body.into_inner();

    match sqlx::query_as!(
        Appointment,
        r#"
        INSERT INTO appointments (
            service_id, business_id, customer_name, 
            customer_email, customer_phone, appointment_time
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
        new_appt.service_id,
        new_appt.business_id,
        new_appt.customer_name,
        new_appt.customer_email,
        new_appt.customer_phone,
        new_appt.appointment_time
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(appointment) => HttpResponse::Created().json(ApiResponse {
            success: true,
            data: Some(appointment),
            message: Some("Appointment created successfully".to_string()),
        }),

        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_foreign_key_violation() {
                bad_request_response("Invalid service_id or business_id provided.".to_string())
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

async fn get_appointment_by_id(path: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let appt_id = path.into_inner();

    match sqlx::query_as!(
        Appointment,
        r#"SELECT * FROM appointments WHERE id = $1"#,
        appt_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(appointment) => {
            let response = ApiResponse {
                success: true,
                data: Some(appointment),
                message: None,
            };
            HttpResponse::Ok().json(response)
        }
        Err(sqlx::Error::RowNotFound) => not_found_response("Appointment not found".to_string()),
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

async fn get_all_appointments(pool: web::Data<PgPool>) -> impl Responder {
    match sqlx::query_as!(Appointment, r#"SELECT * FROM appointments"#)
        .fetch_all(pool.get_ref())
        .await
    {
        Ok(appointment) => HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(appointment),
            message: Some("Appointments retrieved successfully".to_string()),
        }),

        Err(e) => internal_server_error_response(e.to_string()),
    }
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

pub fn appointment_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/appointments")
            .route("", web::post().to(create_appointment))
            .route("", web::get().to(get_all_appointments))
            .route("/{id}", web::get().to(get_appointment_by_id)),
    );
}
