use rocket::{State, form::Form, response::Redirect, http::{Cookie, CookieJar, SameSite}, serde::json::Json, get, post};
use rocket_dyn_templates::{Template, context};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use webauthn_rs::prelude::*;
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::DbConn;
use crate::models::{User, SignupForm, PasskeyRegistrationState};

// Store passkey registration states temporarily with expiration
lazy_static::lazy_static! {
    static ref REGISTRATION_STATES: Arc<Mutex<std::collections::HashMap<String, (PasskeyRegistrationState, u64)>>> = 
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
    
    // Input validation
    if signup_form.username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }
    
    if signup_form.password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    
    if !signup_form.email.contains('@') {
        return Err("Invalid email address".to_string());
    }
    
    // Hash the password
    let password_hash = bcrypt::hash(&signup_form.password, bcrypt::DEFAULT_COST)
        .map_err(|_| "An error occurred during registration".to_string())?;

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
        .map_err(|_| "Service temporarily unavailable. Please try again later.".to_string())?;

    let created: Option<User> = client
        .create("user")
        .content(user)
        .await
        .map_err(|_| "An error occurred during registration".to_string())?;

    if let Some(created_user) = created {
        if let Some(user_id) = &created_user.id {
            // Store user_id and username in secure cookies
            let user_cookie = Cookie::build(("user_id", user_id.to_string()))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/");
            
            let username_cookie = Cookie::build(("username", signup_form.username))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/");
            
            cookies.add(user_cookie);
            cookies.add(username_cookie);
            
            return Ok(Redirect::to("/setup-passkey"));
        }
    }

    Err("An error occurred during registration".to_string())
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
        .ok_or("Authentication required")?
        .value()
        .to_string();
    
    let username = cookies.get("username")
        .ok_or("Authentication required")?
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
        .map_err(|_| "Failed to start passkey registration".to_string())?;

    // Store registration state with expiration (5 minutes)
    let mut states = REGISTRATION_STATES.lock().await;
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 300; // 5 minutes
    
    states.insert(user_id.clone(), (PasskeyRegistrationState {
        user_id,
        reg_state,
    }, expiration));
    
    // Clean up expired states
    states.retain(|_, (_, exp)| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        *exp > now
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
        .ok_or("Authentication required")?
        .value()
        .to_string();

    // Get and validate registration state
    let mut states = REGISTRATION_STATES.lock().await;
    let (state, expiration) = states.remove(&user_id)
        .ok_or("Registration session expired or not found")?;
    
    // Check if expired
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    if expiration < now {
        return Err("Registration session expired".to_string());
    }

    let webauthn_guard = webauthn.lock().await;
    
    let passkey = webauthn_guard
        .finish_passkey_registration(&reg, &state.reg_state)
        .map_err(|_| "Failed to complete passkey registration".to_string())?;

    // Update user with passkey
    let client = db.get_client().await
        .map_err(|_| "Service temporarily unavailable".to_string())?;

    let _updated: Option<User> = client
        .update(("user", user_id.as_str()))
        .merge(json!({
            "passkey": vec![passkey]
        }))
        .await
        .map_err(|_| "Failed to save passkey".to_string())?;

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
