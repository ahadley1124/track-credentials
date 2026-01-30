use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use std::env;

pub async fn init_db() -> Result<Surreal<Client>, Box<dyn std::error::Error>> {
    // Get database configuration from environment variables
    let db_url = env::var("SURREALDB_URL")
        .expect("SURREALDB_URL environment variable must be set");
    
    let db_user = env::var("SURREALDB_USER")
        .expect("SURREALDB_USER environment variable must be set");
    
    let db_pass = env::var("SURREALDB_PASS")
        .expect("SURREALDB_PASS environment variable must be set");
    
    println!("Connecting to SurrealDB...");
    
    // Connect to SurrealDB cloud instance
    let db = Surreal::new::<Ws>(db_url).await?;
    
    // Sign in as root user
    db.signin(Root {
        username: &db_user,
        password: &db_pass,
    })
    .await?;
    
    // Use namespace and database
    db.use_ns("track_credentials").use_db("main").await?;
    
    // Create user table schema
    db.query(
        "DEFINE TABLE user SCHEMAFULL;
         DEFINE FIELD username ON user TYPE string;
         DEFINE FIELD email ON user TYPE string;
         DEFINE FIELD password_hash ON user TYPE string;
         DEFINE FIELD passkey_registered ON user TYPE bool DEFAULT false;
         DEFINE INDEX unique_username ON user COLUMNS username UNIQUE;
         DEFINE INDEX unique_email ON user COLUMNS email UNIQUE;"
    )
    .await?;
    
    println!("Database initialized successfully");
    
    Ok(db)
}
