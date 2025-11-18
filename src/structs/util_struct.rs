use serde::Deserialize;
use serde::{self, Serialize};

#[derive(Deserialize)]
pub struct UploadQuery {
    #[serde(rename = "type")]
    pub upload_type: String, // "profile" or "cover"
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub signed_upload_url: String,
    pub public_url: String,
}
