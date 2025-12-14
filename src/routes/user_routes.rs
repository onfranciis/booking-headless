use std::{collections::HashMap, time::Duration};

use crate::{
    config::Config,
    middlewares::auth_middleware::AuthenticatedUser,
    routes::utils_routes::{
        bad_request_response, conflict_reponse, internal_server_error_response, not_found_response,
    },
    structs::{
        db_struct::{
            Appointment, Auth, AvailabilityRule, Service, SetAvailability, UpdateUser, User,
            UserStatus, UserWithServices,
        },
        response_struct::{ApiResponse, MergedUserProfile},
        util_struct::{
            FreeBusyRequest, FreeBusyRequestItem, FreeBusyResponse, SlotQuery, TimeSlot,
            UploadQuery, UploadResponse,
        },
    },
    utils::{
        auth_utils::{get_gcs_client, get_new_access_token},
        others_utils::local_to_utc,
    },
};
use actix_web::{HttpResponse, Responder, web};
use chrono_tz::Tz;
use gcloud_storage::sign::{SignedURLMethod, SignedURLOptions};
use sqlx::PgPool;
use std::str::FromStr;
use time::{
    Date, Duration as TimeDuration, OffsetDateTime, PrimitiveDateTime, Time,
    format_description::{self, well_known::Rfc3339},
};
use uuid::Uuid;

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

async fn get_user_by_id(path: web::Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    let user_id = path.into_inner();

    let profile = match sqlx::query_as!(User, r#"SELECT * FROM users WHERE id = $1"#, user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(user) => user,

        Err(sqlx::Error::RowNotFound) => return not_found_response("User not found".to_string()),

        Err(e) => return internal_server_error_response(e.to_string()),
    };

    let availability = match sqlx::query_as!(
        AvailabilityRule,
        r#"SELECT * FROM business_availability WHERE user_id = $1"#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rules) => rules,

        Err(e) => return internal_server_error_response(e.to_string()),
    };

    let response = ApiResponse {
        success: true,
        message: None,
        data: Some(MergedUserProfile {
            profile,
            availability,
        }),
    };

    return HttpResponse::ServiceUnavailable().json(response);
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
    match sqlx::query_as!(User, "SELECT * FROM users WHERE is_active = TRUE")
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
    let users_result = sqlx::query_as!(User, r#"SELECT * FROM users WHERE is_active = TRUE"#)
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
        ORDER BY appointment_start_time DESC
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

async fn get_me(user: AuthenticatedUser, pool: web::Data<PgPool>) -> impl Responder {
    let logged_in_user_id = user.user_id;

    let profile = match sqlx::query_as!(
        User,
        r#"SELECT * FROM users WHERE id = $1"#,
        logged_in_user_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(user) => user,

        Err(sqlx::Error::RowNotFound) => return not_found_response("User not found".to_string()),

        Err(e) => return internal_server_error_response(e.to_string()),
    };

    let availability = match sqlx::query_as!(
        AvailabilityRule,
        r#"SELECT * FROM business_availability WHERE user_id = $1"#,
        logged_in_user_id
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rules) => rules,

        Err(e) => return internal_server_error_response(e.to_string()),
    };

    let response = ApiResponse {
        success: true,
        message: None,
        data: Some(MergedUserProfile {
            profile,
            availability,
        }),
    };

    return HttpResponse::ServiceUnavailable().json(response);
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

async fn get_user_upload_url(
    user: AuthenticatedUser,
    query: web::Query<UploadQuery>,
    config: web::Data<Config>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = user.user_id;
    let upload_type = &query.upload_type;

    // Validate the query type
    let (db_column, file_path) = match upload_type.as_str() {
        "cover" => ("cover_image_url", format!("users/{}/cover.jpg", user_id)),

        "profile" => (
            "profile_image_url",
            format!("users/{}/profile.jpg", user_id),
        ),

        _ => {
            return bad_request_response(
                "Invalid upload type. Must be 'profile' or 'cover'".to_string(),
            );
        }
    };

    let client = get_gcs_client(&config).await;
    let bucket_name = &config.gcs_bucket_name;

    // Generate the signed URL for uploading
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

    // Generate the final public URL
    let public_url = format!(
        "https://storage.googleapis.com/{}/{}",
        bucket_name, file_path
    );

    // Update the user's database record
    // We use a dynamic query because we can't use `query_as!` with a dynamic column name
    let query_str = format!("UPDATE users SET {} = $1 WHERE id = $2", db_column);
    if let Err(e) = sqlx::query(&query_str)
        .bind(&public_url)
        .bind(user_id)
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

async fn set_account_status(
    user: AuthenticatedUser,
    body: web::Json<UserStatus>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = user.user_id;
    let status = body.status.unwrap_or(false);

    match sqlx::query_as!(
        User,
        r#"
        UPDATE users 
        SET is_active = $1,
        updated_at = NOW() 
        WHERE id = $2 
        RETURNING *
        "#,
        status,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(updated_user) => {
            let response = ApiResponse {
                success: true,
                data: Some(updated_user),
                message: Some("Account status updated successfully.".to_string()),
            };

            HttpResponse::Ok().json(response)
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

async fn get_available_slots(
    path: web::Path<Uuid>,
    query: web::Query<SlotQuery>,
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    http_client: web::Data<reqwest::Client>,
) -> impl Responder {
    let user_id = path.into_inner();
    let date_str = &query.date;

    let format = match format_description::parse("[year]-[month]-[day]") {
        Ok(f) => f,
        Err(_) => {
            // This would never reach here because of a correct format in the parse above
            return internal_server_error_response("Invalid date format in code!".to_string());
        }
    };

    let requested_date = match Date::parse(date_str, &format) {
        Ok(d) => d,
        Err(_) => return bad_request_response("Invalid date format (YYYY-MM-DD)".to_string()),
    };

    let service = match sqlx::query_as!(
        Service,
        "SELECT * FROM services WHERE id = $1",
        query.service_id
    )
    .fetch_one(pool.get_ref())
    .await
    {
        Ok(s) => s,
        Err(_) => return not_found_response("Service not found.".to_string()),
    };

    let duration_minutes = service.duration_minutes.unwrap_or(30) as i64;
    let weekday = requested_date.weekday().number_from_monday() as i32;

    let rules = match sqlx::query_as!(
        AvailabilityRule,
        "SELECT * FROM business_availability WHERE user_id = $1 AND day_of_week = $2",
        user_id,
        weekday
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(r) => r,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    if rules.is_empty() {
        return HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(Vec::<TimeSlot>::new()),
            message: Some("Closed this day".to_string()),
        });
    }

    let tz: Tz = match Tz::from_str(&rules[0].time_zone) {
        Ok(z) => z,
        Err(_) => return internal_server_error_response("Invalid DB Timezone".to_string()),
    };

    // Fetch appointments for the whole day (in UTC).
    // We construct a generous UTC window to catch all potential overlaps.
    let day_start_naive = PrimitiveDateTime::new(requested_date, Time::MIDNIGHT);
    let day_end_naive = PrimitiveDateTime::new(requested_date.next_day().unwrap(), Time::MIDNIGHT);

    let utc_window_start = local_to_utc(day_start_naive, &tz).unwrap();
    let utc_window_end = local_to_utc(day_end_naive, &tz).unwrap();

    // We will collect ALL unavailable times (DB + Google) into this vector
    let mut blocked_periods: Vec<(OffsetDateTime, OffsetDateTime)> = Vec::new();

    let db_appointments = match sqlx::query_as!(
        Appointment,
        "SELECT * FROM appointments WHERE business_id = $1
        AND appointment_end_time > $2
        AND appointment_start_time < $3",
        user_id,
        utc_window_start,
        utc_window_end
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(a) => a,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    // Add DB appointments to blocked list
    for appt in db_appointments {
        blocked_periods.push((appt.appointment_start_time, appt.appointment_end_time));
    }

    let auth_record = sqlx::query_as!(Auth, "SELECT * FROM auth WHERE user_id = $1", user_id)
        .fetch_optional(pool.get_ref())
        .await
        .unwrap_or(None);

    if let Some(auth) = auth_record {
        if let Some(refresh_token) = auth.refresh_token {
            if let Ok(access_token) =
                get_new_access_token(config, &http_client, refresh_token).await
            {
                let freebusy_request = FreeBusyRequest {
                    time_min: utc_window_start.format(&Rfc3339).unwrap(),
                    time_max: utc_window_end.format(&Rfc3339).unwrap(),
                    items: vec![FreeBusyRequestItem {
                        id: "primary".to_string(),
                    }],
                };

                let freebusy_response = http_client
                    .post("https://www.googleapis.com/calendar/v3/freeBusy")
                    .bearer_auth(access_token)
                    .json(&freebusy_request)
                    .send()
                    .await;

                if let Ok(res) = freebusy_response {
                    if let Ok(parsed) = res.json::<FreeBusyResponse>().await {
                        if let Some(calendar) = parsed.calendars.get("primary") {
                            for busy_slot in &calendar.busy {
                                let start = OffsetDateTime::parse(&busy_slot.start, &Rfc3339).ok();
                                let end = OffsetDateTime::parse(&busy_slot.end, &Rfc3339).ok();

                                if let (Some(s), Some(e)) = (start, end) {
                                    blocked_periods.push((s, e));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Generate Slots And Check Collisions
    let mut available_slots: Vec<TimeSlot> = Vec::new();
    let step = TimeDuration::minutes(30);

    for rule in rules {
        let mut current_open_naive = PrimitiveDateTime::new(requested_date, rule.open_time);
        let current_close_naive = PrimitiveDateTime::new(requested_date, rule.close_time);

        while current_open_naive + TimeDuration::minutes(duration_minutes) <= current_close_naive {
            let slot_end_naive = current_open_naive + TimeDuration::minutes(duration_minutes);

            // Convert to UTC for comparison
            if let Some(slot_start_utc) = local_to_utc(current_open_naive, &tz) {
                if let Some(slot_end_utc) = local_to_utc(slot_end_naive, &tz) {
                    let is_clashing = blocked_periods.iter().any(|(busy_start, busy_end)| {
                        // Overlap Logic: (StartA < EndB) and (EndA > StartB)
                        *busy_start < slot_end_utc && *busy_end > slot_start_utc
                    });

                    if !is_clashing {
                        available_slots.push(TimeSlot {
                            start_time: slot_start_utc.format(&Rfc3339).unwrap(),
                            end_time: slot_end_utc.format(&Rfc3339).unwrap(),
                        });
                    }
                }
            }

            current_open_naive += step;
        }
    }

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(available_slots),
        message: None,
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

async fn set_user_availability(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    body: web::Json<SetAvailability>,
) -> impl Responder {
    let user_id = user.user_id;
    let rules = body.rules.iter();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error_response(e.to_string()),
    };

    // Delete all existing rules for this user
    if let Err(e) = sqlx::query!(
        "DELETE FROM business_availability WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await
    {
        tx.rollback().await.ok();
        return internal_server_error_response(e.to_string());
    }

    let time_format = match time::format_description::parse("[hour]:[minute]:[second]") {
        Ok(format) => format,
        Err(_) => return internal_server_error_response("Internal time format error.".to_string()),
    };

    // Insert the new rules
    for slot in rules {
        let open_time = match Time::parse(&slot.open_time, &time_format) {
            Ok(t) => t,
            Err(_) => {
                return bad_request_response(format!(
                    "Invalid open_time format for {}. Expected HH:MM:SS.",
                    slot.open_time
                ));
            }
        };

        let close_time = match Time::parse(&slot.close_time, &time_format) {
            Ok(t) => t,
            Err(_) => {
                return bad_request_response(format!(
                    "Invalid close_time format for {}. Expected HH:MM:SS.",
                    slot.close_time
                ));
            }
        };

        if let Err(e) = sqlx::query!(
           r#"
            INSERT INTO business_availability (user_id, day_of_week, open_time, close_time, time_zone)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            user_id,
            slot.day_of_week,
            open_time,
            close_time,
            slot.time_zone
        )
        .execute(&mut *tx)
        .await
        {
            tx.rollback().await.ok();
            return internal_server_error_response(e.to_string());
        }
    }

    if let Err(e) = tx.commit().await {
        return internal_server_error_response(e.to_string());
    }

    let response = ApiResponse::<()> {
        success: true,
        data: None,
        message: Some("Availability schedule updated successfully.".to_string()),
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

pub fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/me", web::get().to(get_me))
            .route("/me/upload-url", web::get().to(get_user_upload_url))
            .route("/me/status", web::patch().to(set_account_status))
            .route("/me/availability", web::post().to(set_user_availability))
            .route("/with-services", web::get().to(get_all_users_with_services))
            .route("/{id}/slots", web::get().to(get_available_slots))
            .route(
                "/{id}/appointments",
                web::get().to(get_appointments_for_user),
            )
            .route("/{id}", web::get().to(get_user_by_id))
            .route("/{id}", web::patch().to(update_user))
            .route("", web::get().to(get_all_users)),
    );
}
