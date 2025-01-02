mod middleware;

use actix_web::{post, web, App, HttpServer, Result};
use backend::crypt::{CryptService, EncryptedData};
use std::sync::{Arc, Mutex};
use actix_cors::Cors;
use middleware::ServiceError;

struct AppState {
    crypt_service: Arc<Mutex<CryptService>>,
}

#[post("/encrypt")]
async fn encrypt(
    data: web::Json<String>,
    state: web::Data<AppState>
) -> Result<web::Json<EncryptedData>, ServiceError> {
    let crypt_service = state.crypt_service.lock()
        .map_err(|e| ServiceError::ServiceLockError(e.to_string()))?;
    
    let encrypted = crypt_service.encrypt_data(&data.into_inner())
        .map_err(|e| ServiceError::EncryptionError(e.to_string()))?;
    
    Ok(web::Json(encrypted))
}

#[post("/decrypt")]
async fn decrypt(
    encrypted: web::Json<EncryptedData>,
    state: web::Data<AppState>
) -> Result<web::Json<String>, ServiceError> {
    let crypt_service = state.crypt_service.lock()
        .map_err(|e| ServiceError::ServiceLockError(e.to_string()))?;
    
    let decrypted = crypt_service.decrypt_data(&encrypted.into_inner())
        .map_err(|e| ServiceError::DecryptionError(e.to_string()))?;
    
    Ok(web::Json(decrypted))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let crypt_service = CryptService::new();
    let app_state = web::Data::new(AppState {
        crypt_service: Arc::new(Mutex::new(crypt_service)),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(encrypt)
            .service(decrypt)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
