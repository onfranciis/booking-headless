use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

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
pub struct CreateUser {
    pub username: String,
    pub business_name: String,
    pub email: String,
    pub location: Option<String>,
    pub phone_number: Option<String>,
    pub description: Option<String>,
    pub phone_number_is_whatsapp: Option<bool>,
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
    pub user_id: Uuid,
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
