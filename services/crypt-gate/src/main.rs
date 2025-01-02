use actix_web::{error, post, web, App, HttpResponse, HttpServer};
use serde::{Deserialize, Serialize};
use backend::CryptService;
use std::fmt;
use std::sync::Mutex;
use actix_cors::Cors;

struct AppState {
    crypt_service: Mutex<CryptService>,
}

#[derive(Debug)]
enum CryptError {
    CryptFailed(String),
}

impl fmt::Display for CryptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CryptError::CryptFailed(msg) => write!(f, "Şifreleme hatası: {}", msg),
        }
    }
}

impl error::ResponseError for CryptError {
    fn error_response(&self) -> HttpResponse {
        match self {
            _ => HttpResponse::InternalServerError().json(ErrorResponse {
                error: self.to_string(),
                code: "INTERNAL_ERROR"
            }),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: &'static str,
}

#[derive(Deserialize)]
struct CryptRequest {
    data: String,
}

#[derive(Serialize)]
struct CryptResponse {
    result: String,
}

#[post("/encrypt")]
async fn encrypt(
    req: web::Json<CryptRequest>,
    data: web::Data<AppState>
) -> Result<HttpResponse, CryptError> {
    let crypt_service = data.crypt_service.lock().unwrap();
    
    let encrypted = crypt_service.encrypt_data(&req.data)
        .map_err(|e| CryptError::CryptFailed(e))?;
    
    Ok(HttpResponse::Ok().json(CryptResponse {
        result: encrypted
    }))
}

#[post("/decrypt")]
async fn decrypt(
    req: web::Json<CryptRequest>,
    data: web::Data<AppState>
) -> Result<HttpResponse, CryptError> {
    let crypt_service = data.crypt_service.lock().unwrap();
    
    let decrypted = crypt_service.decrypt_data(&req.data)
        .map_err(|e| CryptError::CryptFailed(e))?;
    
    Ok(HttpResponse::Ok().json(CryptResponse {
        result: decrypted
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let crypt_service = CryptService::new();
    let app_state = web::Data::new(AppState {
        crypt_service: Mutex::new(crypt_service),
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
