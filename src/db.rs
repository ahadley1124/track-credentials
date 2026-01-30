use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use std::sync::Arc;
use tokio::sync::Mutex;

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
            let db = Surreal::new::<Ws>("projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud").await?;
            
            // Sign in with root credentials
            db.signin(Root {
                username: "root",
                password: "root",
            })
            .await?;

            // Use namespace and database
            db.use_ns("track_credentials").use_db("main").await?;

            *guard = Some(db);
        }

        Ok(guard.as_ref().unwrap().clone())
    }
}
