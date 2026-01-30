use rocket::fs::{FileServer, relative};
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::{get, post, launch, routes, State};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use urlencoding::encode;

mod db;

#[derive(Debug)]
enum AppError {
    DatabaseError(String),
    ValidationError(String),
}

impl From<AppError> for Template {
    fn from(error: AppError) -> Self {
        let error_msg = match error {
            AppError::DatabaseError(msg) => format!("Database error: {}", msg),
            AppError::ValidationError(msg) => format!("Validation error: {}", msg),
        };
        Template::render("error", context! {
            error: error_msg
        })
    }
}

#[derive(Debug, Serialize, Deserialize, FromForm)]
struct SignupForm {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: Option<String>,
    username: String,
    email: String,
    password_hash: String,
    passkey_registered: bool,
}

#[get("/")]
fn index() -> Redirect {
    Redirect::to("/signup")
}

#[get("/signup")]
fn signup_page() -> Template {
    Template::render("signup", context! {})
}

#[post("/signup", data = "<form>")]
async fn signup_submit(
    form: Form<SignupForm>,
    db: &State<Surreal<Client>>,
) -> Result<Redirect, Template> {
    // Validate input
    if form.username.is_empty() || form.email.is_empty() || form.password.is_empty() {
        return Err(AppError::ValidationError("All fields are required".to_string()).into());
    }
    
    if form.password.len() < 8 {
        return Err(AppError::ValidationError("Password must be at least 8 characters".to_string()).into());
    }
    
    if !form.email.contains('@') {
        return Err(AppError::ValidationError("Invalid email address".to_string()).into());
    }
    
    // Simple password hashing (in production, use bcrypt or argon2)
    let password_hash = format!("hashed_{}", form.password);
    
    let user = User {
        id: None,
        username: form.username.clone(),
        email: form.email.clone(),
        password_hash,
        passkey_registered: false,
    };
    
    // Create user in database
    let created: Result<Option<User>, _> = db
        .create("user")
        .content(user)
        .await;
    
    match created {
        Ok(Some(_user)) => {
            // Store user ID in session or pass as query parameter
            // For simplicity, we'll redirect directly to passkey setup
            // URL-encode the username to handle special characters
            Ok(Redirect::to(format!("/passkey-setup?username={}", encode(&form.username))))
        }
        Ok(None) => {
            Err(AppError::DatabaseError("Failed to create user".to_string()).into())
        }
        Err(e) => Err(AppError::DatabaseError(format!("Error creating user: {}", e)).into()),
    }
}

#[get("/passkey-setup?<username>")]
fn passkey_setup(username: String) -> Template {
    Template::render("passkey_setup", context! {
        username: username
    })
}

#[derive(FromForm)]
struct PasskeyForm {
    username: String,
}

#[post("/passkey-setup", data = "<form>")]
async fn passkey_complete(
    form: Form<PasskeyForm>,
    db: &State<Surreal<Client>>,
) -> Result<Redirect, Template> {
    // Clone the username to avoid lifetime issues
    let username = form.username.clone();
    
    // Update user to mark passkey as registered
    let result = db
        .query("UPDATE user SET passkey_registered = true WHERE username = $username")
        .bind(("username", username))
        .await;
    
    match result {
        Ok(_) => Ok(Redirect::to("/home")),
        Err(e) => Err(AppError::DatabaseError(format!("Error updating passkey status: {}", e)).into()),
    }
}

#[get("/home")]
fn home() -> Template {
    Template::render("home", context! {
        message: "Welcome to your home page!"
    })
}

#[launch]
async fn rocket() -> rocket::Rocket<rocket::Build> {
    // Initialize database
    let db = db::init_db().await.expect("Failed to initialize database");
    
    rocket::build()
        .manage(db)
        .mount("/", routes![
            index,
            signup_page,
            signup_submit,
            passkey_setup,
            passkey_complete,
            home
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}
