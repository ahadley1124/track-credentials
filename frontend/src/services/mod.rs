use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupResponse {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub username: String,
    pub has_passkey: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub created_at: Option<String>,
    pub last_login: Option<String>,
}

pub async fn signup(username: String, email: String, password: String) -> Result<SignupResponse, String> {
    let request = SignupRequest { username, email, password };
    
    let response = Request::post("http://localhost:8000/api/signup")
        .json(&request)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.ok() {
        response.json::<SignupResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let error = response.json::<ApiError>()
            .await
            .map_err(|e| format!("Error parsing error: {}", e))?;
        Err(error.error)
    }
}

pub async fn login(username: String, password: String) -> Result<LoginResponse, String> {
    let request = LoginRequest { username, password };
    
    let response = Request::post("http://localhost:8000/api/login")
        .json(&request)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.ok() {
        response.json::<LoginResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let error = response.json::<ApiError>()
            .await
            .map_err(|e| format!("Error parsing error: {}", e))?;
        Err(error.error)
    }
}

pub async fn get_current_user() -> Result<User, String> {
    let response = Request::get("http://localhost:8000/api/user/me")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.ok() {
        response.json::<User>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err("Not authenticated".to_string())
    }
}

pub async fn logout() -> Result<(), String> {
    Request::post("http://localhost:8000/api/logout")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    Ok(())
}
