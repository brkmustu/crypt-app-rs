mod error;

use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use backend::crypt::{CryptService, EncryptedData};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::Mutex;
use error::{ProcessError, ProcessResponse};
use actix_web::{web, App, HttpServer, get, Error, HttpRequest, HttpResponse};
use actix_ws::Message as WsMessage;
use std::time::Duration;
use tokio::sync::mpsc;
use std::collections::HashMap;
use actix_cors::Cors;

#[derive(Deserialize, Serialize, Clone, Debug)]
struct CryptMessage {
    id: String,
    operation: String,
    data: String,
}

#[derive(Serialize, Debug)]
struct WebSocketResponse {
    success: bool,
    message_id: String,
    data: Option<String>,
    error: Option<String>,
}

async fn process_message(message: CryptMessage, crypt_service: &CryptService) -> WebSocketResponse {
    println!("İşlenen mesaj: {:?}", message);

    let result = match message.operation.as_str() {
        "encrypt" => encrypt_data(crypt_service, &message.data).await,
        "decrypt" => decrypt_data(crypt_service, &message.data).await,
        _ => Err(ProcessError::FormatError("Geçersiz operasyon".to_string())),
    };

    let response = match result {
        Ok(data) => WebSocketResponse {
            success: true,
            message_id: message.id.clone(),
            data: Some(data),
            error: None,
        },
        Err(err) => WebSocketResponse {
            success: false,
            message_id: message.id.clone(),
            data: None,
            error: Some(err.to_string()),
        },
    };

    println!("Dönüş mesajı: {:?}", response);
    response
}

async fn encrypt_data(crypt_service: &CryptService, data: &str) -> Result<String, ProcessError> {
    let encrypted = crypt_service.encrypt_data(data)
        .map_err(|e| ProcessError::CryptError(e.to_string()))?;
    
    serde_json::to_string(&encrypted)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))
}

async fn decrypt_data(crypt_service: &CryptService, encrypted_str: &str) -> Result<String, ProcessError> {
    let encrypted_data: EncryptedData = serde_json::from_str(encrypted_str)
        .map_err(|e| ProcessError::SerializationError(e.to_string()))?;
    
    crypt_service.decrypt_data(&encrypted_data)
        .map_err(|e| ProcessError::CryptError(e.to_string()))
}

const ENCRYPT_QUEUE: &str = "encrypt_queue";
const DECRYPT_QUEUE: &str = "decrypt_queue";
const RABBITMQ_URL: &str = "amqp://cryptuser:cryptpass@localhost:5672";

#[derive(Clone)]
struct WebSocketManager {
    connections: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<String>>>>,
    pending_messages: Arc<Mutex<Vec<String>>>,
}

impl WebSocketManager {
    fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            pending_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn add_connection(&self, id: String, tx: mpsc::UnboundedSender<String>) {
        let mut connections = self.connections.lock().await;
        connections.insert(id.clone(), tx.clone());
        println!("Yeni WebSocket bağlantısı eklendi: {}", id);

        let mut pending = self.pending_messages.lock().await;
        for message in pending.iter() {
            if let Err(e) = tx.send(message.clone()) {
                println!("Bekleyen mesaj gönderilemedi: {}", e);
            }
        }
        pending.clear();
    }

    async fn broadcast_message(&self, message: String) -> bool {
        let mut retry_count = 0;
        let max_retries = 3;

        while retry_count < max_retries {
            let mut connections = self.connections.lock().await;
            
            if connections.is_empty() {
                println!("Aktif bağlantı yok, bekleniyor... ({}/{})", retry_count + 1, max_retries);
                drop(connections);
                tokio::time::sleep(Duration::from_secs(1)).await;
                retry_count += 1;
                continue;
            }

            let mut success = false;
            connections.retain(|id, tx| {
                match tx.send(message.clone()) {
                    Ok(_) => {
                        println!("Mesaj gönderildi: {}", id);
                        success = true;
                        true
                    },
                    Err(_) => {
                        println!("Kapalı bağlantı siliniyor: {}", id);
                        false
                    }
                }
            });

            if success {
                return true;
            }

            drop(connections);
            tokio::time::sleep(Duration::from_secs(1)).await;
            retry_count += 1;
        }

        let mut pending = self.pending_messages.lock().await;
        pending.push(message);
        println!("Mesaj bekleme kuyruğuna alındı");
        false
    }

    async fn remove_connection(&self, id: &str) {
        let mut connections = self.connections.lock().await;
        if connections.remove(id).is_some() {
            println!("WebSocket bağlantısı silindi: {}", id);
        }
    }
}

#[get("/ws")]
async fn websocket(
    req: HttpRequest,
    body: web::Payload,
    manager: web::Data<Arc<WebSocketManager>>,
) -> Result<HttpResponse, Error> {
    let id = Uuid::new_v4().to_string();
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let (tx, mut rx) = mpsc::unbounded_channel();
    manager.add_connection(id.clone(), tx).await;

    println!("WebSocket bağlantısı başlatıldı: {}", id);

    actix_web::rt::spawn(async move {
        let mut ping_timer = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                Some(msg) = msg_stream.next() => {
                    match msg {
                        Ok(WsMessage::Ping(bytes)) => {
                            if let Err(e) = session.pong(&bytes).await {
                                println!("Ping yanıtı gönderilemedi: {}", e);
                                break;
                            }
                        }
                        Ok(WsMessage::Close(_)) => {
                            println!("WebSocket kapanış isteği alındı: {}", id);
                            break;
                        }
                        Err(e) => {
                            println!("WebSocket hata: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                Some(msg) = rx.recv() => {
                    if let Err(e) = session.text(msg).await {
                        println!("Mesaj gönderilemedi: {}", e);
                        break;
                    }
                }
                _ = ping_timer.tick() => {
                    if let Err(e) = session.ping(b"").await {
                        println!("Ping gönderilemedi: {}", e);
                        break;
                    }
                }
            }
        }

        println!("WebSocket bağlantısı kapandı: {}", id);
        manager.remove_connection(&id).await;
    });

    Ok(response)
}

async fn handle_encrypt_messages(
    mut consumer: lapin::Consumer,
    crypt_service: Arc<CryptService>,
    manager: Arc<WebSocketManager>,
) {
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            if let Ok(message) = serde_json::from_slice::<CryptMessage>(&delivery.data) {
                let response = process_message(message.clone(), &crypt_service).await;
                if let Ok(response_json) = serde_json::to_string(&response) {
                    println!("WebSocket üzerinden gönderiliyor: {}", response_json);
                    
                    let mut retry_count = 0;
                    let max_retries = 3;
                    let mut success = false;

                    while retry_count < max_retries && !success {
                        success = manager.broadcast_message(response_json.clone()).await;
                        if !success {
                            retry_count += 1;
                            if retry_count < max_retries {
                                println!("Yeniden deneniyor ({}/{})", retry_count, max_retries);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
        }
    }
}

async fn handle_decrypt_messages(
    mut consumer: lapin::Consumer,
    crypt_service: Arc<CryptService>,
    manager: Arc<WebSocketManager>,
) {
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            if let Ok(message) = serde_json::from_slice::<CryptMessage>(&delivery.data) {
                let response = process_message(message.clone(), &crypt_service).await;
                if let Ok(response_json) = serde_json::to_string(&response) {
                    println!("WebSocket üzerinden gönderiliyor: {}", response_json);
                    let mut retry_count = 0;
                    let max_retries = 3;
                    let mut success = false;

                    while retry_count < max_retries && !success {
                        success = manager.broadcast_message(response_json.clone()).await;
                        if !success {
                            retry_count += 1;
                            if retry_count < max_retries {
                                println!("Yeniden deneniyor ({}/{})", retry_count, max_retries);
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
                let _ = delivery.ack(BasicAckOptions::default()).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_manager = Arc::new(WebSocketManager::new());
    let ws_manager_data = web::Data::new(ws_manager.clone());

    // WebSocket sunucusu
    let websocket_task = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(ws_manager_data.clone())
            .service(websocket)
    })
    .bind("127.0.0.1:8083")?
    .run();

    // RabbitMQ bağlantısı ve consumer'lar
    let conn = Connection::connect(RABBITMQ_URL, ConnectionProperties::default())
        .await?;
    
    let channel = conn.create_channel().await?;
    let crypt_service = Arc::new(CryptService::new());

    let encrypt_consumer = channel.basic_consume(
        ENCRYPT_QUEUE,
        "encrypt_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;

    let decrypt_consumer = channel.basic_consume(
        DECRYPT_QUEUE,
        "decrypt_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default(),
    ).await?;

    println!("RabbitMQ consumers başlatıldı");

    // Consumer task'ları başlat
    let crypt_service_clone = crypt_service.clone();
    let encrypt_handle = tokio::spawn(handle_encrypt_messages(
        encrypt_consumer,
        crypt_service_clone,
        ws_manager.clone(),
    ));
    
    let crypt_service_clone = crypt_service.clone();
    let decrypt_handle = tokio::spawn(handle_decrypt_messages(
        decrypt_consumer,
        crypt_service_clone,
        ws_manager.clone(),
    ));

    // WebSocket sunucusunu bekle
    websocket_task.await?;

    Ok(())
}
