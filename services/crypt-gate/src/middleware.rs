use std::fmt;
use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use serde_json::json;

#[derive(Debug)]
pub enum ServiceError {
    EncryptionError(String),
    DecryptionError(String),
    ServiceLockError(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServiceError::EncryptionError(msg) => write!(f, "Şifreleme hatası: {}", msg),
            ServiceError::DecryptionError(msg) => write!(f, "Çözme hatası: {}", msg),
            ServiceError::ServiceLockError(msg) => write!(f, "Servis kilidi hatası: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        let (error_message, error_code) = match self {
            ServiceError::EncryptionError(msg) => (msg.to_string(), "ENCRYPTION_ERROR"),
            ServiceError::DecryptionError(msg) => (msg.to_string(), "DECRYPTION_ERROR"),
            ServiceError::ServiceLockError(msg) => (msg.to_string(), "SERVICE_LOCK_ERROR"),
        };

        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(json!({
                "error": error_message,
                "code": error_code
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
} 