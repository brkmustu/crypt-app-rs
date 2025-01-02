use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use backend::CryptService;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize, Serialize)]
struct CryptMessage {
    id: String,
    operation: String,
    data: String,
}

#[derive(Debug)]
enum ProcessError {
    CryptError(String),
    FormatError(String),
}

impl From<ProcessError> for String {
    fn from(error: ProcessError) -> Self {
        match error {
            ProcessError::CryptError(msg) => format!("Şifreleme hatası: {}", msg),
            ProcessError::FormatError(msg) => format!("Format hatası: {}", msg),
        }
    }
}

#[derive(Serialize)]
struct ProcessResponse {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

async fn process_message(message: CryptMessage, crypt_service: &CryptService) -> ProcessResponse {
    let result = match message.operation.as_str() {
        "encrypt" => encrypt_data(crypt_service, &message.data).await,
        "decrypt" => decrypt_data(crypt_service, &message.data).await,
        _ => Err(ProcessError::FormatError("Geçersiz operasyon".to_string())),
    };

    match result {
        Ok(data) => ProcessResponse {
            success: true,
            result: Some(data),
            error: None,
        },
        Err(err) => ProcessResponse {
            success: false,
            result: None,
            error: Some(err.into()),
        },
    }
}

async fn encrypt_data(crypt_service: &CryptService, data: &str) -> Result<String, ProcessError> {
    // RSA ile şifrele
    let encrypted = crypt_service.decrypt_data(data)
        .map_err(|e| ProcessError::CryptError(e))?;

    Ok(encrypted)
}

async fn decrypt_data(crypt_service: &CryptService, data: &str) -> Result<String, ProcessError> {
    // RSA ile çöz
    let decrypted = crypt_service.decrypt_data(data)
        .map_err(|e| ProcessError::CryptError(e))?;

    Ok(decrypted)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "amqp://cryptuser:cryptpass@localhost:5672";
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    
    let crypt_service = Arc::new(CryptService::new());
    
    // Kuyruk oluşturma
    let _ = channel.queue_declare(
        "crypt_queue",
        QueueDeclareOptions::default(),
        FieldTable::default()
    ).await?;
    
    // WebSocket bağlantısı
    let (ws_stream, _) = connect_async("ws://localhost:8081").await?;
    let (ws_sender, _ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));
    
    println!("Servis başlatıldı, mesajlar bekleniyor...");
    
    let consumer = channel.basic_consume(
        "crypt_queue",
        "crypt_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;
    
    let crypt_service_clone = crypt_service.clone();
    let ws_sender_clone = ws_sender.clone();

    consumer.for_each(move |delivery| {
        let crypt_service = crypt_service_clone.clone();
        let ws_sender = ws_sender_clone.clone();

        async move {
            let delivery = match delivery {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Teslimat hatası: {}", e);
                    return;
                }
            };

            let message: CryptMessage = match serde_json::from_slice(&delivery.data) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("JSON ayrıştırma hatası: {}", e);
                    if let Err(e) = delivery.reject(BasicRejectOptions::default()).await {
                        eprintln!("Mesaj reddetme hatası: {}", e);
                    }
                    return;
                }
            };

            let response = process_message(message, &crypt_service).await;
            let response_json = match serde_json::to_string(&response) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("JSON serileştirme hatası: {}", e);
                    return;
                }
            };

            if let Err(e) = ws_sender.lock().await.send(response_json.into()).await {
                eprintln!("WebSocket gönderme hatası: {}", e);
            }

            if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                eprintln!("Onaylama hatası: {}", e);
            }
        }
    }).await;
    
    Ok(())
}
