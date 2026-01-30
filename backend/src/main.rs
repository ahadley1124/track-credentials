#[macro_use] extern crate rocket;

mod db;
mod models;

use rocket::State;
use rocket::response::{status, content};
use rocket::http::{Status, Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::fs::{FileServer, relative};
use rocket_cors::{AllowedOrigins, CorsOptions};
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use webauthn_rs::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use models::{User, UserPasskey, SignupRequest, SignupResponse, LoginRequest, LoginResponse, ApiError};

// Application state
struct AppState {
    db: Surreal<Any>,
    webauthn: Arc<Webauthn>,
    passkey_challenges: Arc<Mutex<HashMap<String, PasskeyRegistration>>>,
}

// API: POST /api/signup
#[post("/api/signup", data = "<signup_req>")]
async fn api_signup(
    signup_req: Json<SignupRequest>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<SignupResponse>, status::Custom<Json<ApiError>>> {
    let data = signup_req.into_inner();
    
    // Validate input
    if data.username.is_empty() || data.email.is_empty() || data.password.is_empty() {
        return Err(status::Custom(
            Status::BadRequest,
            Json(ApiError { error: "All fields are required".to_string() })
        ));
    }
    
    if data.password.len() < 8 {
        return Err(status::Custom(
            Status::BadRequest,
            Json(ApiError { error: "Password must be at least 8 characters".to_string() })
        ));
    }
    
    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(data.password.as_bytes(), &salt)
        .map_err(|_| status::Custom(
            Status::InternalServerError,
            Json(ApiError { error: "Failed to process password".to_string() })
        ))?
        .to_string();
    
    // Create user
    let result: Result<Option<User>, _> = state.db
        .create("users")
        .content(User {
            id: None,
            username: data.username.clone(),
            email: data.email,
            password_hash,
            created_at: None,
            last_login: None,
        })
        .await;
    
    match result {
        Ok(Some(created_user)) => {
            if let Some(user_id) = &created_user.id {
                let user_id_str = user_id.to_string();
                cookies.add(Cookie::new("user_id", user_id_str.clone()));
                cookies.add(Cookie::new("username", data.username.clone()));
                    
                Ok(Json(SignupResponse {
                    user_id: user_id_str,
                    username: data.username,
                }))
            } else {
                Err(status::Custom(
                    Status::InternalServerError,
                    Json(ApiError { error: "Failed to create user".to_string() })
                ))
            }
        },
        Ok(None) => {
            Err(status::Custom(
                Status::InternalServerError,
                Json(ApiError { error: "Failed to create user".to_string() })
            ))
        },
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            let error_msg = if e.to_string().contains("unique") {
                "Username or email already exists"
            } else {
                "Failed to create account"
            };
            
            Err(status::Custom(
                Status::Conflict,
                Json(ApiError { error: error_msg.to_string() })
            ))
        }
    }
}

// API: POST /api/login
#[post("/api/login", data = "<login_req>")]
async fn api_login(
    login_req: Json<LoginRequest>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<LoginResponse>, status::Custom<Json<ApiError>>> {
    let data = login_req.into_inner();
    
    // Find user by username
    let query = format!("SELECT * FROM users WHERE username = '{}'", data.username);
    let mut result = state.db.query(&query).await
        .map_err(|_| status::Custom(
            Status::InternalServerError,
            Json(ApiError { error: "Database error".to_string() })
        ))?;
    
    let users: Vec<User> = result.take(0)
        .map_err(|_| status::Custom(
            Status::Unauthorized,
            Json(ApiError { error: "Invalid credentials".to_string() })
        ))?;
    
    if let Some(user) = users.first() {
        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|_| status::Custom(
                Status::InternalServerError,
                Json(ApiError { error: "Server error".to_string() })
            ))?;
        
        Argon2::default()
            .verify_password(data.password.as_bytes(), &parsed_hash)
            .map_err(|_| status::Custom(
                Status::Unauthorized,
                Json(ApiError { error: "Invalid credentials".to_string() })
            ))?;
        
        // Update last_login
        if let Some(user_id) = &user.id {
            let update_query = format!("UPDATE {} SET last_login = time::now()", user_id);
            let _: Result<surrealdb::Response, _> = state.db.query(update_query).await;
            
            // Set session cookies
            cookies.add(Cookie::new("user_id", user_id.to_string()));
            cookies.add(Cookie::new("username", user.username.clone()));
            
            // Check if user has passkey
            let passkey_query = format!("SELECT * FROM passkeys WHERE user_id = {}", user_id);
            let passkey_result = state.db.query(&passkey_query).await;
            let has_passkey = if let Ok(mut pr) = passkey_result {
                let passkeys: Vec<UserPasskey> = pr.take(0).unwrap_or_default();
                !passkeys.is_empty()
            } else {
                false
            };
            
            Ok(Json(LoginResponse {
                user_id: user_id.to_string(),
                username: user.username.clone(),
                has_passkey,
            }))
        } else {
            Err(status::Custom(
                Status::InternalServerError,
                Json(ApiError { error: "Server error".to_string() })
            ))
        }
    } else {
        Err(status::Custom(
            Status::Unauthorized,
            Json(ApiError { error: "Invalid credentials".to_string() })
        ))
    }
}

// API: POST /api/passkey/register/start
#[post("/api/passkey/register/start")]
async fn api_passkey_register_start(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Json<CreationChallengeResponse>, status::Custom<Json<ApiError>>> {
    let user_id_str = cookies.get("user_id")
        .ok_or(status::Custom(
            Status::Unauthorized,
            Json(ApiError { error: "Not authenticated".to_string() })
        ))?
        .value()
        .to_string();
    
    let username = cookies.get("username")
        .ok_or(status::Custom(
            Status::Unauthorized,
            Json(ApiError { error: "Not authenticated".to_string() })
        ))?
        .value()
        .to_string();
    
    // Create unique user ID for WebAuthn
    let user_unique_id = Uuid::parse_str(&uuid::Uuid::new_v4().to_string())
        .map_err(|_| status::Custom(
            Status::InternalServerError,
            Json(ApiError { error: "Failed to generate user ID".to_string() })
        ))?;
    
    let (ccr, reg_state) = state.webauthn
        .start_passkey_registration(
            user_unique_id,
            &username,
            &username,
            None,
        )
        .map_err(|e| {
            eprintln!("WebAuthn error: {:?}", e);
            status::Custom(
                Status::InternalServerError,
                Json(ApiError { error: "Failed to start passkey registration".to_string() })
            )
        })?;
    
    // Store registration state
    let mut challenges = state.passkey_challenges.lock().await;
    challenges.insert(user_id_str.clone(), reg_state);
    
    Ok(Json(ccr))
}

// API: POST /api/passkey/register/finish
#[post("/api/passkey/register/finish", data = "<reg_response>")]
async fn api_passkey_register_finish(
    reg_response: Json<RegisterPublicKeyCredential>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Status, status::Custom<Json<ApiError>>> {
    let user_id_str = cookies.get("user_id")
        .ok_or(status::Custom(
            Status::Unauthorized,
            Json(ApiError { error: "Not authenticated".to_string() })
        ))?
        .value()
        .to_string();
    
    // Get stored registration state
    let mut challenges = state.passkey_challenges.lock().await;
    let reg_state = challenges.remove(&user_id_str)
        .ok_or(status::Custom(
            Status::BadRequest,
            Json(ApiError { error: "No registration in progress".to_string() })
        ))?;
    
    // Finish registration
    let passkey = state.webauthn
        .finish_passkey_registration(&reg_response, &reg_state)
        .map_err(|e| {
            eprintln!("WebAuthn verification error: {:?}", e);
            status::Custom(
                Status::BadRequest,
                Json(ApiError { error: "Failed to verify passkey".to_string() })
            )
        })?;
    
    // Store passkey in database
    let _result: Result<Option<UserPasskey>, _> = state.db
        .create("passkeys")
        .content(UserPasskey {
            id: None,
            user_id: surrealdb::sql::thing(&user_id_str).unwrap(),
            credential_id: passkey.cred_id().to_vec(),
            public_key: serde_json::to_vec(&passkey).unwrap(),
            counter: 0,
            created_at: None,
        })
        .await;
    
    Ok(Status::Ok)
}

// API: GET /api/user/me
#[get("/api/user/me")]
async fn api_user_me(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Json<User>, status::Custom<Json<ApiError>>> {
    let user_id_str = cookies.get("user_id")
        .ok_or(status::Custom(
            Status::Unauthorized,
            Json(ApiError { error: "Not authenticated".to_string() })
        ))?
        .value()
        .to_string();
    
    let user: Option<User> = state.db.select(("users", &user_id_str)).await
        .map_err(|_| status::Custom(
            Status::InternalServerError,
            Json(ApiError { error: "Database error".to_string() })
        ))?;
    
    user.ok_or(status::Custom(
        Status::NotFound,
        Json(ApiError { error: "User not found".to_string() })
    )).map(Json)
}

// API: POST /api/logout
#[post("/api/logout")]
fn api_logout(cookies: &CookieJar<'_>) -> Status {
    cookies.remove(Cookie::from("user_id"));
    cookies.remove(Cookie::from("username"));
    Status::Ok
}

// Catch-all route to serve index.html for client-side routing
#[get("/<_..>", rank = 100)]
async fn spa_index() -> Option<content::RawHtml<String>> {
    std::fs::read_to_string(relative!("../frontend/dist/index.html"))
        .ok()
        .map(content::RawHtml)
}

#[launch]
async fn rocket() -> _ {
    // Initialize database
    let db = match db::init_db().await {
        Ok(db) => {
            println!("✓ Database connected successfully");
            db
        },
        Err(e) => {
            eprintln!("✗ Failed to connect to database: {}", e);
            panic!("Database connection failed");
        }
    };
    
    // Initialize WebAuthn
    let rp_id = "localhost";
    let rp_origin = Url::parse("http://localhost:8000")
        .expect("Invalid URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid configuration");
    let webauthn = Arc::new(builder.build().expect("Invalid configuration"));
    
    // CORS configuration
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec!["Get", "Post", "Put", "Delete"]
                .into_iter()
                .map(|s| s.parse().unwrap())
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .expect("CORS configuration error");
    
    rocket::build()
        .manage(AppState {
            db,
            webauthn,
            passkey_challenges: Arc::new(Mutex::new(HashMap::new())),
        })
        .attach(cors)
        .mount("/", routes![
            api_signup,
            api_login,
            api_passkey_register_start,
            api_passkey_register_finish,
            api_user_me,
            api_logout,
        ])
        .mount("/", FileServer::from(relative!("../frontend/dist")))
        .mount("/", routes![spa_index])
}
