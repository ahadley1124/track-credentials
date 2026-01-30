use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub async fn init_db() -> Result<Surreal<Client>, Box<dyn std::error::Error>> {
    // Connect to SurrealDB cloud instance
    let db = Surreal::new::<Ws>("projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud")
        .await?;
    
    // Sign in as root user
    db.signin(Root {
        username: "cloud",
        password: "ThisIsCloud",
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
