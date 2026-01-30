use rocket::{Build, Rocket, routes};
use rocket_dyn_templates::Template;
use std::sync::Arc;
use tokio::sync::Mutex;
use webauthn_rs::prelude::*;
use url::Url;

mod db;
mod routes;
mod models;

use db::DbConn;

#[rocket::main]
async fn main() {
    let _ = rocket().launch().await;
}

fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![
            routes::index,
            routes::signup_get,
            routes::signup_post,
            routes::setup_passkey_get,
            routes::setup_passkey_register_start,
            routes::setup_passkey_register_finish,
            routes::home
        ])
        .manage(DbConn::new())
        .manage(init_webauthn())
}

fn init_webauthn() -> Arc<Mutex<Webauthn>> {
    let rp_id = "localhost";
    let rp_origin = Url::parse("http://localhost:8000")
        .expect("Invalid URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid configuration");
    
    Arc::new(Mutex::new(
        builder.build().expect("Invalid webauthn configuration")
    ))
}
