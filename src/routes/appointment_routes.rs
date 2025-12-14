use crate::{
    config::Config,
    routes::utils_routes::{
        bad_request_response, expectation_failed_response, internal_server_error_response,
        not_found_response,
    },
    structs::{
        db_struct::{
            Appointment, Auth, AvailabilityRule, CreateAppointment, GoogleCalendarEvent,
            GoogleEventAttendee, GoogleEventDateTime, Service,
        },
        response_struct::ApiResponse,
    },
    utils::{auth_utils::get_new_access_token, others_utils::convert_to_local_primitive},
};
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use time::{Duration, format_description::well_known::Rfc3339};
use uuid::Uuid;

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn create_appointment(
    config: web::Data<Config>,
    pool: web::Data<PgPool>,
    body: web::Json<CreateAppointment>,
    http_client: web::Data<reqwest::Client>,
) -> impl Responder {
    let new_appt = body.into_inner();

    // Start Transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return internal_server_error_response(format!("Failed to start transaction: {}", e));
        }
    };

    // Fetch Service to know the duration
    let service = match sqlx::query_as!(
        Service,
        r#"SELECT * FROM services WHERE id = $1"#,
        new_appt.service_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(s) => s,

        Err(sqlx::Error::RowNotFound) => {
            tx.rollback().await.ok();
            return bad_request_response("Invalid service_id.".to_string());
        }

        Err(e) => {
            tx.rollback().await.ok();
            return internal_server_error_response(e.to_string());
        }
    };

    // Fetch Business Auth for Google Token
    let auth_record = match sqlx::query_as!(
        Auth,
        r#"SELECT * FROM auth WHERE user_id = $1"#,
        new_appt.business_id
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(a) => a,

        Err(sqlx::Error::RowNotFound) => {
            tx.rollback().await.ok();
            return bad_request_response("Business not found or not authenticated.".to_string());
        }

        Err(e) => {
            tx.rollback().await.ok();
            return internal_server_error_response(e.to_string());
        }
    };

    // Check Business Active Status
    let is_active = sqlx::query_scalar!(
        r#"SELECT is_active as "is_active!: bool" FROM users WHERE id = $1"#,
        new_appt.business_id
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap_or(false);

    if !is_active {
        tx.rollback().await.ok();

        return bad_request_response(
            "This business is not currently accepting appointments.".to_string(),
        );
    }

    // Calculate Time And Check Availability
    let start_time = new_appt.appointment_start_time;
    let duration = service.duration_minutes.unwrap_or(30);
    let end_time = start_time + Duration::minutes(duration as i64);
    let weekday = start_time.weekday().number_from_monday() as i32;

    let rules = match sqlx::query_as!(
        AvailabilityRule,
        r#"SELECT * FROM business_availability WHERE user_id = $1 AND day_of_week = $2"#,
        new_appt.business_id,
        weekday
    )
    .fetch_all(&mut *tx)
    .await
    {
        Ok(r) => r,

        Err(e) => {
            tx.rollback().await.ok();

            return internal_server_error_response(e.to_string());
        }
    };

    if rules.is_empty() {
        tx.rollback().await.ok();

        return bad_request_response("Business is closed on this day.".to_string());
    }

    // Conversion and Comparison Logic
    let first_rule = &rules[0];

    let local_start = match convert_to_local_primitive(start_time, &first_rule.time_zone) {
        Ok(dt) => dt.time(),

        Err(e) => {
            tx.rollback().await.ok();

            return internal_server_error_response(e);
        }
    };

    let local_end = match convert_to_local_primitive(end_time, &first_rule.time_zone) {
        Ok(dt) => dt.time(),

        Err(e) => {
            tx.rollback().await.ok();

            return internal_server_error_response(e);
        }
    };

    let business_is_available = rules
        .iter()
        .any(|rule| local_start >= rule.open_time && local_end <= rule.close_time);

    if !business_is_available {
        tx.rollback().await.ok();

        return bad_request_response("Requested slot is outside operating hours.".to_string());
    }

    // Save Appointment to Database
    let appointment = match sqlx::query_as!(
        Appointment,
        r#"
        INSERT INTO appointments (
            service_id, business_id, customer_name, customer_email, 
            customer_phone, appointment_start_time, notes, appointment_end_time
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
        new_appt.service_id,
        new_appt.business_id,
        new_appt.customer_name,
        new_appt.customer_email,
        new_appt.customer_phone,
        start_time,
        new_appt.notes,
        end_time
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(appt) => appt,

        Err(e) => {
            tx.rollback().await.ok();
            return internal_server_error_response(e.to_string());
        }
    };

    // Google Calendar Sync
    if let Some(refresh_token) = auth_record.refresh_token {
        let access_token = match get_new_access_token(config, &http_client, refresh_token).await {
            Ok(token) => token,

            Err(e) => {
                tx.rollback().await.ok();

                return internal_server_error_response(format!(
                    "Failed to refresh Google token: {}",
                    e
                ));
            }
        };

        // Format Dates Safely
        let start_fmt = match start_time.format(&Rfc3339) {
            Ok(formatted_string) => formatted_string,

            Err(e) => {
                tx.rollback().await.ok();

                return internal_server_error_response(e.to_string());
            }
        };

        let end_fmt = match end_time.format(&Rfc3339) {
            Ok(formatted_string) => formatted_string,

            Err(e) => {
                tx.rollback().await.ok();

                return internal_server_error_response(e.to_string());
            }
        };

        // Build Event
        let notes_str = new_appt
            .notes
            .as_deref()
            .map(|n| format!("\n\nNotes: {}", n))
            .unwrap_or("N/A".to_string());

        let event = GoogleCalendarEvent {
            summary: format!(
                "Appointment Scheduled: {} for {}",
                service.service_name, new_appt.customer_name
            ),
            description: format!(
                "Service: {}\nCustomer Phone: {}\nCustomer Email: {}\nNote: {}",
                service.service_name,
                new_appt.customer_phone.as_deref().unwrap_or("N/A"),
                new_appt.customer_email.as_deref().unwrap_or("N/A"),
                notes_str
            ),
            start: GoogleEventDateTime {
                date_time: start_fmt,
                time_zone: "UTC".to_string(),
            },
            end: GoogleEventDateTime {
                date_time: end_fmt,
                time_zone: "UTC".to_string(),
            },
            attendees: vec![
                // Add the customer as an attendee so they get an invite
                GoogleEventAttendee {
                    email: new_appt.customer_email.unwrap_or_default(),
                },
            ],
        };

        // Send to Google Calendar
        let google_res = http_client
            .post("https://www.googleapis.com/calendar/v3/calendars/primary/events?sendUpdates=all")
            .bearer_auth(access_token)
            .json(&event)
            .send()
            .await;

        match google_res {
            Ok(res) if res.status().is_success() => {}

            Ok(res) => {
                // Google returned an error, maybe a 400 - Bad Request
                let err_text = res.text().await.unwrap_or_default();

                tx.rollback().await.ok();

                return internal_server_error_response(format!(
                    "Google Calendar API Error: {}",
                    err_text
                ));
            }

            Err(e) => {
                tx.rollback().await.ok();

                return internal_server_error_response(format!("Failed to contact Google: {}", e));
            }
        }
    } else {
        let message = format!(
            "Info: Business {} has no Google Calendar connected.",
            new_appt.business_id
        );

        return expectation_failed_response(message);
    }

    if let Err(e) = tx.commit().await {
        return internal_server_error_response(format!("Failed to commit transaction: {}", e));
    }

    let response = ApiResponse {
        success: true,
        data: Some(appointment),
        message: Some("Appointment created and synced.".to_string()),
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
