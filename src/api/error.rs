use std::fmt::Debug;
use std::fmt::Display;

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use serde_json::json;

#[derive(Debug, Clone)]
/// The errors returned by this API
pub enum ApiError {
    /// The requested amount in milisats is less than our minSendable
    AmountTooSmall,
    /// Something went wrong with our backend. Usually it's our connection with
    /// phoenixd that had problems
    BackendError,
    /// User sent us a non-alphanumeric string. Check for those because we do fs
    /// operations with user-provided data
    InvalidString,
    /// Requested an unknown user
    UnknownUser,
    /// User sent us a string that's not ascii-encoded
    NonAsciiString,
    /// Requested username is too long
    StringTooLong,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ApiError::BackendError => {
                StatusCode::from_u16(500).expect("hardcoded value should be valid")
            }
            ApiError::AmountTooSmall => {
                StatusCode::from_u16(400).expect("hardcoded value should be valid")
            }
            ApiError::InvalidString => {
                StatusCode::from_u16(400).expect("hardcoded value should be valid")
            }
            ApiError::StringTooLong => {
                StatusCode::from_u16(400).expect("hardcoded value should be valid")
            }
            ApiError::NonAsciiString => {
                StatusCode::from_u16(400).expect("hardcoded value should be valid")
            }
            ApiError::UnknownUser => {
                StatusCode::from_u16(404).expect("hardcoded value should be valid")
            }
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            ApiError::AmountTooSmall => HttpResponse::BadRequest()
                .json(json!({"status": "ERROR", "reason": "amount too small"})),
            ApiError::BackendError => HttpResponse::InternalServerError().into(),
            ApiError::UnknownUser => HttpResponse::NotFound()
                .json(json!({"status": "ERROR", "reason": "user not found"})),
            ApiError::NonAsciiString => HttpResponse::BadRequest()
                .json(json!({"status": "ERROR", "reason": "non-ascii char found in string"})),
            ApiError::StringTooLong => HttpResponse::BadRequest()
                .json(json!({"status": "ERROR", "reason": "provided string is too long"})),
            ApiError::InvalidString => HttpResponse::BadRequest()
                .json(json!({"status": "ERROR", "reason": "invalid char found in string"})),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(_value: reqwest::Error) -> Self {
        ApiError::BackendError
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_value: serde_json::Error) -> Self {
        ApiError::BackendError
    }
}
