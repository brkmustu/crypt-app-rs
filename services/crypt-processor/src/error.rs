use std::fmt;
use serde::Serialize;

#[derive(Debug)]
pub enum ProcessError {
    CryptError(String),
    FormatError(String),
    QueueError(String),
    WebSocketError(String),
    SerializationError(String),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessError::CryptError(msg) => write!(f, "Şifreleme hatası: {}", msg),
            ProcessError::FormatError(msg) => write!(f, "Format hatası: {}", msg),
            ProcessError::QueueError(msg) => write!(f, "Kuyruk hatası: {}", msg),
            ProcessError::WebSocketError(msg) => write!(f, "WebSocket hatası: {}", msg),
            ProcessError::SerializationError(msg) => write!(f, "Serileştirme hatası: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

#[derive(Serialize)]
pub struct ProcessResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<ErrorDetail>,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub message: String,
    pub code: &'static str,
}

impl ProcessResponse {
    pub fn success(result: String) -> Self {
        Self {
            success: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(error: ProcessError) -> Self {
        let (message, code) = match &error {
            ProcessError::CryptError(msg) => (msg.clone(), "CRYPT_ERROR"),
            ProcessError::FormatError(msg) => (msg.clone(), "FORMAT_ERROR"),
            ProcessError::QueueError(msg) => (msg.clone(), "QUEUE_ERROR"),
            ProcessError::WebSocketError(msg) => (msg.clone(), "WEBSOCKET_ERROR"),
            ProcessError::SerializationError(msg) => (msg.clone(), "SERIALIZATION_ERROR"),
        };

        Self {
            success: false,
            result: None,
            error: Some(ErrorDetail { message, code }),
        }
    }
} 