use actix_web::{web, App, HttpServer, post, HttpResponse, http};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};
use std::env;
use std::sync::Mutex;
use backend::crypt::CryptService;

struct AppState {
    crypt_service: Mutex<CryptService>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    public_key: String,  // RSA public key
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
}

#[post("/login")]
async fn login(
    req: web::Json<LoginRequest>,
    data: web::Data<AppState>
) -> HttpResponse {
    if req.username == "admin" && req.password == "password123" {
        let now = Utc::now();
        let claims = Claims {
            sub: req.username.clone(),
            exp: (now + Duration::days(1)).timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(env::var("JWT_SECRET").unwrap_or("your-secret-key".to_string()).as_bytes()),
        ).unwrap();

        let crypt_service = data.crypt_service.lock().unwrap();
        let public_key = crypt_service.get_public_key();

        HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "http://localhost:5173"))
            .append_header(("Access-Control-Allow-Credentials", "true"))
            .json(LoginResponse { 
                token,
                public_key
            })
    } else {
        HttpResponse::Unauthorized()
            .append_header(("Access-Control-Allow-Origin", "http://localhost:5173"))
            .append_header(("Access-Control-Allow-Credentials", "true"))
            .json(serde_json::json!({
                "error": "Invalid username or password"
            }))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let crypt_service = CryptService::new();
    let app_state = web::Data::new(AppState {
        crypt_service: Mutex::new(crypt_service),
    });

    HttpServer::new(move || {
        let cors = Cors::permissive()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .expose_headers(["content-type", "authorization"])
            .max_age(3600)
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(login)
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}
