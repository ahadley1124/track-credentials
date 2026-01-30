use rocket::{State, form::Form, response::Redirect, http::{Cookie, CookieJar}, serde::json::Json, get, post};
use rocket_dyn_templates::{Template, context};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use webauthn_rs::prelude::*;
use uuid::Uuid;

use crate::db::DbConn;
use crate::models::{User, SignupForm, PasskeyRegistrationState};

// Store passkey registration states temporarily (in production, use Redis or similar)
lazy_static::lazy_static! {
    static ref REGISTRATION_STATES: Arc<Mutex<std::collections::HashMap<String, PasskeyRegistrationState>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
}

#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to("/signup")
}

#[get("/signup")]
pub async fn signup_get() -> Template {
    Template::render("signup", context! {})
}

#[post("/signup", data = "<form>")]
pub async fn signup_post(
    form: Form<SignupForm>,
    db: &State<DbConn>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, String> {
    let signup_form = form.into_inner();
    
    // Hash the password
    let password_hash = bcrypt::hash(&signup_form.password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Password hashing failed: {}", e))?;

    // Create user
    let user = User {
        id: None,
        username: signup_form.username.clone(),
        email: signup_form.email,
        password_hash,
        passkey: None,
    };

    // Save to database
    let client = db.get_client().await
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let created: Option<User> = client
        .create("user")
        .content(user)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))?;

    if let Some(created_user) = created {
        if let Some(user_id) = &created_user.id {
            // Store user_id in cookie
            cookies.add(Cookie::new("user_id", user_id.to_string()));
            cookies.add(Cookie::new("username", signup_form.username));
            
            return Ok(Redirect::to("/setup-passkey"));
        }
    }

    Err("Failed to create user".to_string())
}

#[get("/setup-passkey")]
pub async fn setup_passkey_get(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    if let Some(username) = cookies.get("username") {
        Ok(Template::render("setup_passkey", context! {
            username: username.value()
        }))
    } else {
        Err(Redirect::to("/signup"))
    }
}

#[post("/setup-passkey/register/start")]
pub async fn setup_passkey_register_start(
    webauthn: &State<Arc<Mutex<Webauthn>>>,
    cookies: &CookieJar<'_>,
) -> Result<Json<CreationChallengeResponse>, String> {
    let user_id = cookies.get("user_id")
        .ok_or("User not found")?
        .value()
        .to_string();
    
    let username = cookies.get("username")
        .ok_or("Username not found")?
        .value()
        .to_string();

    let user_unique_id = Uuid::new_v4();
    
    let webauthn_guard = webauthn.lock().await;
    
    let (ccr, reg_state) = webauthn_guard
        .start_passkey_registration(
            user_unique_id,
            &username,
            &username,
            None,
        )
        .map_err(|e| format!("Failed to start registration: {}", e))?;

    // Store registration state
    let mut states = REGISTRATION_STATES.lock().await;
    states.insert(user_id.clone(), PasskeyRegistrationState {
        user_id,
        reg_state,
    });

    Ok(Json(ccr))
}

#[post("/setup-passkey/register/finish", data = "<reg>")]
pub async fn setup_passkey_register_finish(
    reg: Json<RegisterPublicKeyCredential>,
    webauthn: &State<Arc<Mutex<Webauthn>>>,
    db: &State<DbConn>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, String> {
    let user_id = cookies.get("user_id")
        .ok_or("User not found")?
        .value()
        .to_string();

    // Get registration state
    let mut states = REGISTRATION_STATES.lock().await;
    let state = states.remove(&user_id)
        .ok_or("Registration state not found")?;

    let webauthn_guard = webauthn.lock().await;
    
    let passkey = webauthn_guard
        .finish_passkey_registration(&reg, &state.reg_state)
        .map_err(|e| format!("Failed to finish registration: {}", e))?;

    // Update user with passkey
    let client = db.get_client().await
        .map_err(|e| format!("Database connection failed: {}", e))?;

    let _updated: Option<User> = client
        .update(("user", user_id.as_str()))
        .merge(json!({
            "passkey": vec![passkey]
        }))
        .await
        .map_err(|e| format!("Failed to update user with passkey: {}", e))?;

    Ok(Redirect::to("/home"))
}

#[get("/home")]
pub async fn home(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    if let Some(username) = cookies.get("username") {
        Ok(Template::render("home", context! {
            username: username.value()
        }))
    } else {
        Err(Redirect::to("/signup"))
    }
}
