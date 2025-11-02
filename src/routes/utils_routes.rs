use actix_web::{HttpResponse, Responder, get, web};

use crate::structs::response_struct::ApiResponse;
#[get("/")]
pub async fn home() -> impl Responder {
    Ok::<web::Json<ApiResponse<()>>, actix_web::Error>(web::Json(ApiResponse::<()> {
        success: true,
        data: None,
        message: Some("Yeah, you're home!".to_string()),
    }))
}

pub async fn not_found() -> impl Responder {
    Ok::<web::Json<ApiResponse<()>>, actix_web::Error>(web::Json(ApiResponse::<()> {
        success: false,
        data: None,
        message: Some("404 Not Found".to_string()),
    }))
}

pub fn internal_server_error_response(message: String) -> HttpResponse {
    eprintln!("Internal Server Error: {}", message);

    HttpResponse::InternalServerError().json(ApiResponse::<()> {
        message: Some("Something went wrong on our end".to_string()),
        data: None,
        success: false,
    })
}
