use crate::database::connection::Database;
use crate::models::AuditLog;
use anyhow::Result;
use chrono;
use serde_json;

impl Database {
    /// Logs an audit event
    pub async fn log_audit_event(&self, skill_id: &str, action: &str, details: &serde_json::Value) -> Result<()> {
        let _: Vec<AuditLog> = self.db.create("audit_logs")
            .content(serde_json::json!({
                "skill_id": format!("ai_skills:{}", skill_id),
                "action": action,
                "details": details
            }))
            .await?;
        
        Ok(())
    }
    
    /// Gets audit logs for a specific skill
    pub async fn get_audit_logs_for_skill(&self, skill_id: &str) -> Result<Vec<AuditLog>> {
        let logs: Vec<AuditLog> = self.db.query(
            "SELECT * FROM audit_logs WHERE skill_id = $skill_id ORDER BY timestamp DESC"
        )
        .bind(("skill_id", format!("ai_skills:{}", skill_id)))
        .await?
        .check()?
        .take(0)?;
        Ok(logs)
    }
    
    /// Gets recent audit logs
    pub async fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<AuditLog>> {
        let logs: Vec<AuditLog> = self.db.query(
            "SELECT * FROM audit_logs ORDER BY timestamp DESC LIMIT $limit"
        )
        .bind(("limit", limit as i64))
        .await?
        .check()?
        .take(0)?;
        Ok(logs)
    }

    /// Cleans up old audit logs older than specified days
    pub async fn cleanup_old_logs(&self, days: i64) -> Result<usize> {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days);
        let result: Vec<(i64,)> = self.db.query(
            "DELETE FROM audit_logs WHERE timestamp < $cutoff RETURN AFTER"
        )
        .bind(("cutoff", cutoff_date))
        .await?
        .check()?
        .take(0)?;

        let deleted_count = result.first().map(|r| r.0 as usize).unwrap_or(0);
        Ok(deleted_count)
    }
}
