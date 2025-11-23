use serde::Serialize;

use crate::structs::db_struct::{AvailabilityRule, User};

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct MergedUserProfile {
    pub profile: User,
    pub availability: Vec<AvailabilityRule>,
}
