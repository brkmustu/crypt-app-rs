mod middleware;

use actix_web::{http, post, web, App, HttpResponse, HttpServer};
use backend::crypt::EncryptedData;
use std::sync::Arc;
use actix_cors::Cors;
use middleware::ServiceError;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable, BasicProperties};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct QueueMessage {
    id: String,
    operation: String,
    data: String,
}

struct AppState {
    amqp_channel: Arc<lapin::Channel>,
}

const ENCRYPT_QUEUE: &str = "encrypt_queue";
const DECRYPT_QUEUE: &str = "decrypt_queue";
const RABBITMQ_URL: &str = "amqp://cryptuser:cryptpass@localhost:5672";

#[post("/encrypt")]
async fn encrypt(
    data: web::Json<String>,
    state: web::Data<AppState>
) -> Result<HttpResponse, ServiceError> {
    let message_id = Uuid::new_v4().to_string();
    
    let queue_message = QueueMessage {
        id: message_id.clone(),
        operation: "encrypt".to_string(),
        data: data.into_inner(),
    };

    let payload = serde_json::to_string(&queue_message)
        .map_err(|e| ServiceError::SerializationError(e.to_string()))?;

    state.amqp_channel.basic_publish(
        "",
        ENCRYPT_QUEUE,
        BasicPublishOptions::default(),
        payload.as_bytes(),
        BasicProperties::default(),
    )
    .await
    .map_err(|e| ServiceError::QueueError(e.to_string()))?;

    Ok(HttpResponse::Accepted().json(json!({
        "message_id": message_id,
        "status": "processing"
    })))
}

#[post("/decrypt")]
async fn decrypt(
    encrypted: web::Json<EncryptedData>,
    state: web::Data<AppState>
) -> Result<HttpResponse, ServiceError> {
    let message_id = Uuid::new_v4().to_string();
    
    let queue_message = QueueMessage {
        id: message_id.clone(),
        operation: "decrypt".to_string(),
        data: serde_json::to_string(&encrypted.into_inner())
            .map_err(|e| ServiceError::SerializationError(e.to_string()))?,
    };

    let payload = serde_json::to_string(&queue_message)
        .map_err(|e| ServiceError::SerializationError(e.to_string()))?;

    state.amqp_channel.basic_publish(
        "",
        DECRYPT_QUEUE,
        BasicPublishOptions::default(),
        payload.as_bytes(),
        BasicProperties::default(),
    )
    .await
    .map_err(|e| ServiceError::QueueError(e.to_string()))?;

    Ok(HttpResponse::Accepted().json(json!({
        "message_id": message_id,
        "status": "processing"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = Connection::connect(RABBITMQ_URL, ConnectionProperties::default())
        .await
        .expect("RabbitMQ bağlantısı başarısız");
    
    let channel = conn.create_channel()
        .await
        .expect("Kanal oluşturulamadı");

    // Her işlem için ayrı kuyruk tanımlama
    for queue in [ENCRYPT_QUEUE, DECRYPT_QUEUE] {
        channel.queue_declare(
            queue,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Kuyruk oluşturulamadı");
    }

    let app_state = web::Data::new(AppState {
        amqp_channel: Arc::new(channel),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(encrypt)
            .service(decrypt)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
