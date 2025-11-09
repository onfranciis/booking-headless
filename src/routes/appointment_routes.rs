use crate::{
    routes::utils_routes::{
        bad_request_response, internal_server_error_response, not_found_response,
    },
    structs::{
        db_struct::{
            Appointment, Auth, CreateAppointment, GoogleCalendarEvent, GoogleEventAttendee,
            GoogleEventDateTime,
        },
        response_struct::ApiResponse,
    },
    utils::auth_utils::get_new_access_token,
};
use actix_web::{HttpResponse, Responder, web};
use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

async fn create_appointment(
    pool: web::Data<PgPool>,
    body: web::Json<CreateAppointment>,
) -> impl Responder {
    let new_appt = body.into_inner();

    // Start a transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    // Save the appointment to OUR database first
    let appointment = match sqlx::query_as!(
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
    .fetch_one(&mut *tx)
    .await
    {
        Ok(appointment) => appointment,

        Err(sqlx::Error::Database(db_err)) => {
            tx.rollback().await.ok();

            if db_err.is_foreign_key_violation() {
                return bad_request_response("Invalid service_id or business_id.".to_string());
            } else {
                return internal_server_error_response(db_err.to_string());
            }
        }

        Err(e) => {
            tx.rollback().await.ok();
            return internal_server_error_response(e.to_string());
        }
    };

    // Get the business's Google Refresh Token
    let auth_record = match sqlx::query_as!(
        Auth,
        r#"SELECT * FROM auth WHERE user_id = $1"#,
        new_appt.business_id
    )
    .fetch_one(&mut *tx) // Continue using the transaction
    .await
    {
        Ok(auth) => auth,

        Err(_) => {
            tx.rollback().await.ok();

            return internal_server_error_response(
                "Could not find auth credentials for this business.".to_string(),
            );
        }
    };

    // Call Google Calendar API to create the event
    if let Some(refresh_token) = auth_record.refresh_token {
        let http_client = reqwest::Client::new();

        // Get a new Access Token from Google
        let access_token = match get_new_access_token(&http_client, refresh_token).await {
            Ok(token) => token,

            Err(e) => {
                tx.rollback().await.ok();

                return internal_server_error_response(e);
            }
        };

        // Build the Calendar Event
        // (Assuming a 30-minute duration for now)
        let start_time = new_appt.appointment_time;
        let end_time = start_time + Duration::minutes(30);

        let event = GoogleCalendarEvent {
            summary: format!("Appointment with {}", new_appt.customer_name),
            description: format!(
                "Customer Phone: {}\nCustomer Email: {}",
                new_appt.customer_phone.as_deref().unwrap_or("N/A"),
                new_appt.customer_email.as_deref().unwrap_or("N/A")
            ),
            start: GoogleEventDateTime {
                date_time: start_time.to_rfc3339(),
                time_zone: "UTC".to_string(),
            },
            end: GoogleEventDateTime {
                date_time: end_time.to_rfc3339(),
                time_zone: "UTC".to_string(),
            },
            attendees: vec![
                // Add the customer as an attendee so they get an invite
                GoogleEventAttendee {
                    email: new_appt.customer_email.unwrap_or_default(),
                },
            ],
        };

        // Send the event to Google
        let res = http_client
            .post("https://www.googleapis.com/calendar/v3/calendars/primary/events?sendUpdates=all")
            .bearer_auth(access_token)
            .json(&event)
            .send()
            .await;

        if let Err(e) = res {
            tx.rollback().await.ok();
            return internal_server_error_response(format!(
                "Failed to create Google Calendar event: {}",
                e
            ));
        }
    } else {
        println!(
            "Business {} has no refresh token, skipping calendar sync.",
            new_appt.business_id
        );
    }

    // Commit our local transaction
    if let Err(e) = tx.commit().await {
        return internal_server_error_response(e.to_string());
    }

    // Return success
    let response = ApiResponse {
        success: true,
        data: Some(appointment),
        message: Some("Appointment created successfully".to_string()),
    };
    HttpResponse::Created().json(response)
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
