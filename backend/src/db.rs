use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub async fn init_db() -> Result<Surreal<Any>, Box<dyn std::error::Error>> {
    // Try connecting to SurrealDB cloud
    println!("Connecting to SurrealDB...");
    let connection_url = "wss://projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud";
    
    let db = surrealdb::engine::any::connect(connection_url).await
        .map_err(|e| format!("Failed to connect to {}: {}", connection_url, e))?;
    
    println!("Selecting namespace and database...");
    // Use namespace and database
    db.use_ns("track_credentials").use_db("main").await
        .map_err(|e| format!("Failed to select namespace/database: {}", e))?;
    
    println!("Authenticating with SurrealDB...");
    // Sign in with credentials
    db.signin(Root {
        username: "cloud",
        password: "ThisIsCloud",
    })
    .await
    .map_err(|e| format!("Authentication failed: {}", e))?;
    
    println!("Initializing database schema...");
    // Initialize tables
    init_schema(&db).await?;
    
    Ok(db)
}

async fn init_schema(db: &Surreal<Any>) -> Result<(), Box<dyn std::error::Error>> {
    // Create users table with schema
    db.query(
        "DEFINE TABLE IF NOT EXISTS users SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS username ON users TYPE string ASSERT string::is::alphanum($value);
         DEFINE FIELD IF NOT EXISTS email ON users TYPE string ASSERT string::is::email($value);
         DEFINE FIELD IF NOT EXISTS password_hash ON users TYPE string;
         DEFINE FIELD IF NOT EXISTS created_at ON users TYPE datetime DEFAULT time::now();
         DEFINE FIELD IF NOT EXISTS last_login ON users TYPE option<datetime>;
         DEFINE INDEX IF NOT EXISTS unique_username ON users FIELDS username UNIQUE;
         DEFINE INDEX IF NOT EXISTS unique_email ON users FIELDS email UNIQUE;"
    )
    .await?;
    
    // Create passkeys table
    db.query(
        "DEFINE TABLE IF NOT EXISTS passkeys SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS user_id ON passkeys TYPE record<users>;
         DEFINE FIELD IF NOT EXISTS credential_id ON passkeys TYPE bytes;
         DEFINE FIELD IF NOT EXISTS public_key ON passkeys TYPE bytes;
         DEFINE FIELD IF NOT EXISTS counter ON passkeys TYPE number;
         DEFINE FIELD IF NOT EXISTS created_at ON passkeys TYPE datetime DEFAULT time::now();
         DEFINE INDEX IF NOT EXISTS unique_credential ON passkeys FIELDS credential_id UNIQUE;"
    )
    .await?;
    
    println!("Database schema initialized successfully");
    Ok(())
}
