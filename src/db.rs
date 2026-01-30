use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::env;

pub struct DbConn {
    pub client: Arc<Mutex<Option<Surreal<Client>>>>,
}

impl DbConn {
    pub fn new() -> Self {
        DbConn {
            client: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_client(&self) -> Result<Surreal<Client>, Box<dyn std::error::Error>> {
        let mut guard = self.client.lock().await;
        
        if guard.is_none() {
            // Get database credentials from environment or use defaults
            let db_url = env::var("SURREAL_URL")
                .unwrap_or_else(|_| "projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud".to_string());
            let db_user = env::var("SURREAL_USER").unwrap_or_else(|_| "root".to_string());
            let db_pass = env::var("SURREAL_PASS").unwrap_or_else(|_| "root".to_string());
            
            let connection_url = format!("wss://{}", db_url);
            
            let db = Surreal::new::<Ws>(&connection_url).await?;
            
            // Sign in with credentials
            db.signin(Root {
                username: &db_user,
                password: &db_pass,
            })
            .await?;

            // Use namespace and database
            db.use_ns("track_credentials").use_db("main").await?;

            *guard = Some(db);
        }

        Ok(guard.as_ref().unwrap().clone())
    }
}
