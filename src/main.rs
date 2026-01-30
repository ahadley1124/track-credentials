use rocket::fs::{FileServer, relative};
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::{get, post, launch, routes, State};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

mod db;

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
) -> Result<Redirect, String> {
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
    let created: Result<Vec<User>, _> = db
        .create("user")
        .content(user)
        .await;
    
    match created {
        Ok(users) => {
            if let Some(_user) = users.first() {
                // Store user ID in session or pass as query parameter
                // For simplicity, we'll redirect directly to passkey setup
                Ok(Redirect::to(format!("/passkey-setup?username={}", form.username)))
            } else {
                Err("Failed to create user".to_string())
            }
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

#[get("/passkey-setup?<username>")]
fn passkey_setup(username: String) -> Template {
    Template::render("passkey_setup", context! {
        username: username
    })
}

#[post("/passkey-setup", data = "<username>")]
async fn passkey_complete(
    username: String,
    db: &State<Surreal<Client>>,
) -> Result<Redirect, String> {
    // Update user to mark passkey as registered
    let _updated = db
        .query("UPDATE user SET passkey_registered = true WHERE username = $username")
        .bind(("username", username))
        .await;
    
    Ok(Redirect::to("/home"))
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
