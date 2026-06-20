use surrealdb::engine::local::Mem;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use anyhow::Result;

/// Database connection wrapper
pub struct Database {
    pub db: Surreal<Mem>,
}

impl Database {
    /// Creates a new database instance
    pub async fn new() -> Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        
        // Sign in as root user
        db.signin(Root {
            username: "root",
            password: "root",
        }).await?;
        
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