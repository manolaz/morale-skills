use crate::database::connection::Database;
use crate::models::{AISkill, RiskFinding, RiskType, Severity};
use anyhow::Result;
use chrono;
use serde_json;
use std::collections::HashMap;

impl Database {
    // === Skill Operations ===
    
    /// Saves a skill to the database
    pub async fn save_skill(&self, mut skill: AISkill) -> Result<AISkill> {
        // Ensure created_at is set
        if skill.created_at.is_none() {
            skill.created_at = Some(chrono::Utc::now());
        }
        
        let created_skill: AISkill = self.db.create("ai_skills")
            .content(skill)
            .await?
            .ok_or(anyhow::anyhow!("Failed to create skill record"))?;
        
        // Log the creation
        self.log_audit_event(&created_skill.id, "skill_created", &serde_json::json!({
            "name": &created_skill.name,
            "file_path": &created_skill.file_path
        })).await?;
        
        Ok(created_skill)
    }
    
    /// Updates an existing skill
    pub async fn update_skill(&self, id: &str, skill: AISkill) -> Result<AISkill> {
        let updated_skill: AISkill = self.db.update(("ai_skills", id))
            .content(skill)
            .await?
            .ok_or(anyhow::anyhow!("Failed to update skill record"))?;
        
        self.log_audit_event(id, "skill_updated", &serde_json::json!({
            "name": &updated_skill.name
        })).await?;
        
        Ok(updated_skill)
    }
    
    /// Retrieves a skill by ID
    pub async fn get_skill(&self, id: &str) -> Result<Option<AISkill>> {
        let records = self.db.select(("ai_skills", id)).await?;
        Ok(records.into_iter().next())
    }
    
    /// Retrieves all skills
    pub async fn get_all_skills(&self) -> Result<Vec<AISkill>> {
        let skills: Vec<AISkill> = self.db.select("ai_skills").await?;
        Ok(skills)
    }
    
    /// Gets skills by status
    pub async fn get_skills_by_status(&self, status: &str) -> Result<Vec<AISkill>> {
        let skills: Vec<AISkill> = self.db.query("SELECT * FROM ai_skills WHERE status = $status")
            .bind(("status", status))
            .await?
            .take(0)?;
        Ok(skills)
    }
    
    /// Gets skills by risk type
    pub async fn get_skills_by_risk_type(&self, risk_type: &RiskType) -> Result<Vec<AISkill>> {
        let sql = format!(
            "SELECT * FROM ai_skills WHERE array::exists(risks[*], {{ risk_type: $risk_type }})"
        );
        let skills: Vec<AISkill> = self.db.query(sql)
            .bind(("risk_type", risk_type.to_string()))
            .await?
            .take(0)?;
        Ok(skills)
    }
    
    /// Gets skills by severity level
    pub async fn get_skills_by_severity(&self, severity: &Severity) -> Result<Vec<AISkill>> {
        let sql = format!(
            "SELECT * FROM ai_skills WHERE array::exists(risks[*], {{ severity: $severity }})"
        );
        let skills: Vec<AISkill> = self.db.query(sql)
            .bind(("severity", severity.to_string()))
            .await?
            .take(0)?;
        Ok(skills)
    }
    
    /// Deletes a skill by ID
    pub async fn delete_skill(&self, id: &str) -> Result<bool> {
        let deleted: Option<AISkill> = self.db.delete(("ai_skills", id)).await?;
        let was_deleted = deleted.is_some();
        
        if was_deleted {
            self.log_audit_event(id, "skill_deleted", &serde_json::json!({})).await?;
        }
        
        Ok(was_deleted)
    }
    
    // === Risk Finding Operations ===
    
    /// Adds a risk finding to a skill
    pub async fn add_risk_finding(&self, skill_id: &str, finding: RiskFinding) -> Result<RiskFinding> {
        // First get the current skill
        let mut skill = self.get_skill(skill_id).await?
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_id))?;
        
        // Add the finding to the skill's risks
        skill.risks.push(finding.clone());
        
        // Update the skill in the database
        self.db.update(("ai_skills", skill_id))
            .content(skill)
            .await?
            .ok_or(anyhow::anyhow!("Failed to update skill with new finding"))?;
        
        // Also store the finding separately for easier querying
        let stored_finding: RiskFinding = self.db.create("risk_findings")
            .content(serde_json::json!({
                "skill_id": format!("ai_skills:{}", skill_id),
                "risk_type": finding.risk_type,
                "severity": finding.severity,
                "description": finding.description,
                "evidence": finding.evidence,
                "location": finding.location.unwrap_or_default()
            }))
            .await?
            .ok_or(anyhow::anyhow!("Failed to create risk finding record"))?;
        
        self.log_audit_event(skill_id, "finding_added", &serde_json::json!({
            "risk_type": &stored_finding.risk_type,
            "severity": &stored_finding.severity
        })).await?;
        
        Ok(stored_finding)
    }
    
    /// Gets all risk findings for a specific skill
    pub async fn get_findings_for_skill(&self, skill_id: &str) -> Result<Vec<RiskFinding>> {
        let findings: Vec<RiskFinding> = self.db.query(
            "SELECT * FROM risk_findings WHERE skill_id = $skill_id ORDER BY timestamp DESC"
        )
        .bind(("skill_id", format!("ai_skills:{}", skill_id)))
        .await?
        .take(0)?;
        Ok(findings)
    }
    
    /// Gets all risk findings filtered by risk type
    pub async fn get_findings_by_risk_type(&self, risk_type: &RiskType) -> Result<Vec<RiskFinding>> {
        let findings: Vec<RiskFinding> = self.db.query(
            "SELECT * FROM risk_findings WHERE risk_type = $risk_type ORDER BY timestamp DESC"
        )
        .bind(("risk_type", risk_type.to_string()))
        .await?
        .take(0)?;
        Ok(findings)
    }
    
    /// Gets all risk findings filtered by severity
    pub async fn get_findings_by_severity(&self, severity: &Severity) -> Result<Vec<RiskFinding>> {
        let findings: Vec<RiskFinding> = self.db.query(
            "SELECT * FROM risk_findings WHERE severity = $severity ORDER BY timestamp DESC"
        )
        .bind(("severity", severity.to_string()))
        .await?
        .take(0)?;
        Ok(findings)
    }
    
    // === Summary and Analytics ===
    
    /// Gets a summary of risks by type
    pub async fn get_risk_summary(&self) -> Result<std::collections::HashMap<String, usize>> {
        let mut summary = std::collections::HashMap::new();
        
        // Get count of each risk type
        let results: Vec<(String, i64)> = self.db.query(
            r#"
            SELECT 
                risk_type,
                count() as count
            FROM risk_findings
            GROUP BY risk_type
            "#,
        ).await?.take(0)?;
        
        for (risk_type, count) in results {
            summary.insert(risk_type, count as usize);
        }
        
        Ok(summary)
    }
    
    /// Gets a summary of risks by severity
    pub async fn get_severity_summary(&self) -> Result<std::collections::HashMap<String, usize>> {
        let mut summary = std::collections::HashMap::new();
        
        // Get count of each severity level
        let results: Vec<(String, i64)> = self.db.query(
            r#"
            SELECT 
                severity,
                count() as count
            FROM risk_findings
            GROUP BY severity
            "#,
        ).await?.take(0)?;
        
        for (severity, count) in results {
            summary.insert(severity, count as usize);
        }
        
        Ok(summary)
    }
    
    /// Gets a summary of skills by status
    pub async fn get_status_summary(&self) -> Result<std::collections::HashMap<String, usize>> {
        let mut summary = std::collections::HashMap::new();
        
        // Get count of each status
        let results: Vec<(String, i64)> = self.db.query(
            r#"
            SELECT 
                status,
                count() as count
            FROM ai_skills
            GROUP BY status
            "#,
        ).await?.take(0)?;
        
        for (status, count) in results {
            summary.insert(status, count as usize);
        }
        
        Ok(summary)
    }
    
    /// Gets risk statistics by skill
    pub async fn get_risk_stats_by_skill(&self) -> Result<Vec<(String, usize)>> {
        let stats: Vec<(String, usize)> = self.db.query(
            r#"
            SELECT 
                name,
                array::len(risks) as risk_count
            FROM ai_skills
            ORDER BY risk_count DESC
            "#,
        ).await?.take(0)?;
        
        Ok(stats)
    }
    
    // === Audit Log Operations ===
    
    /// Logs an audit event
    pub async fn log_audit_event(&self, skill_id: &str, action: &str, details: &serde_json::Value) -> Result<()> {
        let _: Vec<crate::models::AuditLog> = self.db.create("audit_logs")
            .content(serde_json::json!({
                "skill_id": format!("ai_skills:{}", skill_id),
                "action": action,
                "details": details
            }))
            .await?;
        
        Ok(())
    }
    
    /// Gets audit logs for a specific skill
    pub async fn get_audit_logs_for_skill(&self, skill_id: &str) -> Result<Vec<crate::models::AuditLog>> {
        let logs: Vec<crate::models::AuditLog> = self.db.query(
            "SELECT * FROM audit_logs WHERE skill_id = $skill_id ORDER BY timestamp DESC"
        )
        .bind(("skill_id", format!("ai_skills:{}", skill_id)))
        .await?
        .take(0)?;
        Ok(logs)
    }
    
    /// Gets recent audit logs
    pub async fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<crate::models::AuditLog>> {
        let logs: Vec<crate::models::AuditLog> = self.db.query(
            "SELECT * FROM audit_logs ORDER BY timestamp DESC LIMIT $limit"
        )
        .bind(("limit", limit as i64))
        .await?
        .take(0)?;
        Ok(logs)
    }
}