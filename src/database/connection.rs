use surrealdb::engine::local::{Mem, Db};
use surrealdb::opt::auth::Root;
use surrealdb::opt::Config;
use surrealdb::Surreal;
use anyhow::Result;

/// Database connection wrapper
pub struct Database {
    pub db: Surreal<Db>,
}

impl Database {
    /// Creates a new database instance
    pub async fn new() -> Result<Self> {
        let root = Root {
            username: "root",
            password: "root",
        };
        let config = Config::new()
            .user(root.clone())
            .capabilities(Capabilities::all());

        let db = Surreal::new::<Mem>(config).await?;
        
        // Sign in as root user
        db.signin(root).await?;
        
        // Select the namespace and database
        db.use_ns("morale").use_db("skills").await?;
        
        Ok(Database { db })
    }
    
    /// Initializes the database schema
    pub async fn init_schema(&self) -> Result<()> {
        use crate::database::schema::define_tables;
        define_tables(self).await?;
        Ok(())
    }
}