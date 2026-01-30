use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;
use rocket::form::FromForm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub passkey: Option<Vec<Passkey>>,
}

#[derive(FromForm)]
pub struct SignupForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasskeyRegistrationState {
    pub user_id: String,
    pub reg_state: PasskeyRegistration,
}
