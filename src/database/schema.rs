use crate::database::connection::Database;
use anyhow::Result;

/// Defines all database tables and their schemas
pub async fn define_tables(db: &Database) -> Result<()> {
    db.db.query(
        r#"
        DEFINE TABLE ai_skills SCHEMAFULL
            PERMISSIONS FOR select, create, update, delete WHERE id = $auth.id;
        
        DEFINE FIELD name ON TABLE ai_skills TYPE string ASSERT string::len($value) > 0;
        DEFINE FIELD description ON TABLE ai_skills TYPE option<string>;
        DEFINE FIELD file_path ON TABLE ai_skills TYPE string ASSERT string::len($value) > 0;
        DEFINE FIELD created_at ON TABLE ai_skills TYPE datetime VALUE $before OR time::now();
        DEFINE FIELD risks ON TABLE ai_skills TYPE array;
        DEFINE FIELD status ON TABLE ai_skills TYPE string DEFAULT "pending";
        
        DEFINE TABLE risk_findings SCHEMAFULL;
        DEFINE FIELD skill_id ON TABLE risk_findings TYPE record<ai_skills>;
        DEFINE FIELD risk_type ON TABLE risk_findings TYPE string;
        DEFINE FIELD severity ON TABLE risk_findings TYPE string;
        DEFINE FIELD description ON TABLE risk_findings TYPE string;
        DEFINE FIELD evidence ON TABLE risk_findings TYPE string;
        DEFINE FIELD location ON TABLE risk_findings TYPE string;
        DEFINE FIELD timestamp ON TABLE risk_findings TYPE datetime VALUE time::now();
        
        DEFINE TABLE audit_logs SCHEMAFULL;
        DEFINE FIELD skill_id ON TABLE audit_logs TYPE record<ai_skills>;
        DEFINE FIELD action ON TABLE audit_logs TYPE string;
        DEFINE FIELD details ON TABLE audit_logs TYPE object;
        DEFINE FIELD timestamp ON TABLE audit_logs TYPE datetime VALUE time::now();
        
        DEFINE INDEX idx_skill_name ON TABLE ai_skills FIELDS name;
        DEFINE INDEX idx_skill_created ON TABLE ai_skills FIELDS created_at;
        DEFINE INDEX idx_skill_status ON TABLE ai_skills FIELDS status;
        DEFINE INDEX idx_risk_type ON TABLE risk_findings FIELDS risk_type;
        DEFINE INDEX idx_risk_severity ON TABLE risk_findings FIELDS severity;
        DEFINE INDEX idx_risk_skill ON TABLE risk_findings FIELDS skill_id;
        DEFINE INDEX idx_log_skill ON TABLE audit_logs FIELDS skill_id;
        DEFINE INDEX idx_log_action ON TABLE audit_logs FIELDS action;
        "#,
    ).await?.check()?;
    
    Ok(())
}

/// Drops all tables (useful for testing)
#[allow(dead_code)]
pub async fn drop_tables(db: &Database) -> Result<()> {
    db.db.query(
        r#"
        REMOVE TABLE ai_skills;
        REMOVE TABLE risk_findings;
        REMOVE TABLE audit_logs;
        "#,
    ).await?.check()?;
    
    Ok(())
}

/// Clears all data while preserving schema
#[allow(dead_code)]
pub async fn clear_data(db: &Database) -> Result<()> {
    db.db.query(
        r#"
        DELETE ai_skills;
        DELETE risk_findings;
        DELETE audit_logs;
        "#,
    ).await?.check()?;
    
    Ok(())
}