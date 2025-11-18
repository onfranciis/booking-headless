use crate::structs::response_struct::ApiResponse;
use actix_web::{HttpRequest, HttpResponse, error};

pub fn path_error_handler(err: error::PathError, _req: &HttpRequest) -> error::Error {
    let error_response = ApiResponse::<()> {
        success: false,
        data: None,
        message: Some(format!("Invalid input in URL: {}", err.to_string())),
    };

    let http_response = HttpResponse::BadRequest().json(error_response);

    error::InternalError::from_response(err, http_response).into()
}

pub fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    let error_message = match &err {
        error::JsonPayloadError::Deserialize(json_err) => {
            format!("Invalid JSON in request body: {}", json_err.to_string())
        }
        _ => format!("Invalid request: {}", err.to_string()),
    };

    let error_response = ApiResponse::<()> {
        success: false,
        data: None,
        message: Some(error_message),
    };

    let http_response = HttpResponse::BadRequest().json(error_response);

    error::InternalError::from_response(err, http_response).into()
}

pub fn query_error_handler(err: error::QueryPayloadError, _req: &HttpRequest) -> error::Error {
    let error_message = match &err {
        error::QueryPayloadError::Deserialize(query_err) => {
            format!("Invalid query in request url: {}", query_err.to_string())
        }
        _ => format!("Invalid request: {}", err.to_string()),
    };

    let error_response = ApiResponse::<()> {
        success: false,
        data: None,
        message: Some(error_message),
    };

    let http_response = HttpResponse::BadRequest().json(error_response);

    error::InternalError::from_response(err, http_response).into()
}
