use crate::database::connection::Database;
use crate::models::{AISkill, RiskFinding, RiskType, Severity, AuditLog};
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
            .check()?
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
            .check()?
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
            .check()?
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
        .check()?
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
        .check()?
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
        .check()?
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
        ).await?.check()?.take(0)?;
        
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
        ).await?.check()?.take(0)?;
        
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
        ).await?.check()?.take(0)?;
        
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
        ).await?.check()?.take(0)?;
        
        Ok(stats)
    }
    
    // === Utility Functions ===
    
    /// Counts total skills in the database
    pub async fn count_skills(&self) -> Result<usize> {
        let result: Vec<(i64,)> = self.db.query("SELECT count() FROM ai_skills").await?.check()?.take(0)?;
        Ok(result.first().map(|r| r.0 as usize).unwrap_or(0))
    }

    /// Counts total risk findings in the database
    pub async fn count_findings(&self) -> Result<usize> {
        let result: Vec<(i64,)> = self.db.query("SELECT count() FROM risk_findings").await?.check()?.take(0)?;
        Ok(result.first().map(|r| r.0 as usize).unwrap_or(0))
    }

    /// Counts total audit logs in the database
    pub async fn count_audit_logs(&self) -> Result<usize> {
        let result: Vec<(i64,)> = self.db.query("SELECT count() FROM audit_logs").await?.check()?.take(0)?;
        Ok(result.first().map(|r| r.0 as usize).unwrap_or(0))
    }

    /// Checks if a skill exists by ID
    pub async fn skill_exists(&self, id: &str) -> Result<bool> {
        let result: Vec<AISkill> = self.db.query("SELECT * FROM ai_skills WHERE id = $id")
            .bind(("id", id))
            .await?
            .check()?
            .take(0)?;
        Ok(!result.is_empty())
    }

    /// Gets skills with high severity risks
    pub async fn get_high_risk_skills(&self) -> Result<Vec<AISkill>> {
        let skills: Vec<AISkill> = self.db.query(
            r#"
            SELECT * FROM ai_skills 
            WHERE array::some(risks[*].severity, "critical") 
               OR array::some(risks[*].severity, "high")
            "#
        ).await?.check()?.take(0)?;
        Ok(skills)
    }

    /// Gets skills sorted by risk count
    pub async fn get_skills_by_risk_count(&self, descending: bool) -> Result<Vec<AISkill>> {
        let order = if descending { "DESC" } else { "ASC" };
        let query = format!("SELECT * FROM ai_skills ORDER BY array::len(risks) {}", order);
        let skills: Vec<AISkill> = self.db.query(query).await?.check()?.take(0)?;
        Ok(skills)
    }

    /// Searches skills by name or description
    pub async fn search_skills(&self, query: &str) -> Result<Vec<AISkill>> {
        let search_query = format!(
            "SELECT * FROM ai_skills WHERE name CONTAINS $query OR description CONTAINS $query"
        );
        let skills: Vec<AISkill> = self.db.query(search_query)
            .bind(("query", query))
            .await?
            .check()?
            .take(0)?;
        Ok(skills)
    }

    /// Gets skills that were audited within a time range
    pub async fn get_skills_by_time_range(
        &self, 
        start_time: chrono::DateTime<chrono::Utc>, 
        end_time: chrono::DateTime<chrono::Utc>
    ) -> Result<Vec<AISkill>> {
        let skills: Vec<AISkill> = self.db.query(
            "SELECT * FROM ai_skills WHERE created_at >= $start AND created_at <= $end"
        )
        .bind(("start", start_time))
        .bind(("end", end_time))
        .await?
        .check()?
        .take(0)?;
        Ok(skills)
    }

    /// Aggregates risk statistics by risk type and severity
    pub async fn get_detailed_risk_analysis(&self) -> Result<HashMap<String, HashMap<String, usize>>> {
        let mut analysis = HashMap::new();
        
        // Get all risk findings grouped by type and severity
        let results: Vec<(String, String, i64)> = self.db.query(
            r#"
            SELECT 
                risk_type,
                severity,
                count() as count
            FROM risk_findings
            GROUP BY risk_type, severity
            "#
        ).await?.check()?.take(0)?;
        
        for (risk_type, severity, count) in results {
            analysis.entry(risk_type)
                .or_insert_with(HashMap::new)
                .insert(severity, count as usize);
        }
        
        Ok(analysis)
    }

    /// Gets risk trends over time
    pub async fn get_risk_trends(&self) -> Result<Vec<(String, i64)>> {
        let trends: Vec<(String, i64)> = self.db.query(
            r#"
            SELECT 
                time::day(timestamp) as day,
                count() as count
            FROM risk_findings
            GROUP BY time::day(timestamp)
            ORDER BY day
            "#
        ).await?.check()?.take(0)?;
        Ok(trends)
    }

    /// Exports all data as JSON for backup purposes
    pub async fn export_all_data(&self) -> Result<serde_json::Value> {
        let skills = self.get_all_skills().await?;
        let findings: Vec<RiskFinding> = self.db.select("risk_findings").await?;
        let logs: Vec<AuditLog> = self.db.select("audit_logs").await?;

        Ok(serde_json::json!({
            "skills": skills,
            "findings": findings,
            "logs": logs
        }))
    }

    /// Imports data from JSON backup
    pub async fn import_data(&self, data: serde_json::Value) -> Result<()> {
        if let Some(skills_array) = data.get("skills").and_then(|v| v.as_array()) {
            for skill_value in skills_array {
                if let Ok(skill) = serde_json::from_value::<AISkill>(skill_value.clone()) {
                    // Try to update if exists, otherwise create
                    match self.get_skill(&skill.id).await? {
                        Some(_) => {
                            self.db.update(("ai_skills", &skill.id))
                                .content(skill)
                                .await?;
                        },
                        None => {
                            self.db.create("ai_skills")
                                .content(skill)
                                .await?;
                        }
                    }
                }
            }
        }

        if let Some(findings_array) = data.get("findings").and_then(|v| v.as_array()) {
            for finding_value in findings_array {
                if let Ok(finding) = serde_json::from_value::<RiskFinding>(finding_value.clone()) {
                    self.db.create("risk_findings")
                        .content(finding)
                        .await?;
                }
            }
        }

        if let Some(logs_array) = data.get("logs").and_then(|v| v.as_array()) {
            for log_value in logs_array {
                if let Ok(log) = serde_json::from_value::<AuditLog>(log_value.clone()) {
                    self.db.create("audit_logs")
                        .content(log)
                        .await?;
                }
            }
        }

        Ok(())
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
        .check()?
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
        .check()?
        .take(0)?;
        Ok(logs)
    }
}