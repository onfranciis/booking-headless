use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                    AUTH                                    */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
#[derive(Serialize, FromRow)]
pub struct Auth {
    pub id: Uuid,
    pub user_id: Uuid,
    pub google_id: String,

    #[serde(skip)]
    pub refresh_token: Option<String>,

    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct GoogleCode {
    pub code: String,
}

#[derive(Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String, // The unique Google ID (provider_id)
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: Uuid, // Our database user.id
    pub exp: i64,
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                    USER                                    */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

#[derive(Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub business_name: String,
    pub email: String,
    pub location: Option<String>,
    pub phone_number: Option<String>,
    pub cover_image_url: Option<String>,
    pub profile_image_url: Option<String>,
    pub description: Option<String>,
    pub is_verified: Option<bool>,
    pub google_is_connected: Option<bool>,
    pub phone_number_is_whatsapp: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub business_name: Option<String>,
    pub email: Option<String>,
    pub location: Option<String>,
    pub phone_number: Option<String>,
    pub description: Option<String>,
    pub phone_number_is_whatsapp: Option<bool>,
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                  SERVICES                                  */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

#[derive(Serialize, FromRow)]
pub struct Service {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service_name: String,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub duration_minutes: Option<i32>,
    pub category: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateService {
    pub service_name: String,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub duration_minutes: Option<i32>,
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateService {
    pub service_name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub duration_minutes: Option<i32>,
    pub category: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct UserWithServices {
    pub user: User,
    pub services: Vec<Service>,
}

/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                APPOINTMENTS                                */
/* -------------------------------------------------------------------------- */
/* -------------------------------------------------------------------------- */
/*                                      -                                     */
/* -------------------------------------------------------------------------- */

#[derive(Serialize, FromRow)]
pub struct Appointment {
    pub id: Uuid,
    pub service_id: Uuid,
    pub business_id: Uuid,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub appointment_time: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct CreateAppointment {
    pub service_id: Uuid,
    pub business_id: Uuid,
    pub customer_name: String,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub appointment_time: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct GoogleCalendarEvent {
    pub summary: String,
    pub description: String,
    pub start: GoogleEventDateTime,
    pub end: GoogleEventDateTime,
    pub attendees: Vec<GoogleEventAttendee>,
}

#[derive(Serialize)]
pub struct GoogleEventDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: String,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}

#[derive(Serialize)]
pub struct GoogleEventAttendee {
    pub email: String,
}
