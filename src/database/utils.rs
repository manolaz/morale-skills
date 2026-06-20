use crate::database::connection::Database;
use crate::models::{AISkill, RiskFinding, RiskType, Severity, AuditLog};
use anyhow::Result;
use std::collections::HashMap;

/// Utility functions for database operations
impl Database {
    /// Counts total skills in the database
    pub async fn count_skills(&self) -> Result<usize> {
        let result: Vec<(i64,)> = self.db.query("SELECT count() FROM ai_skills").await?.take(0)?;
        Ok(result.first().map(|r| r.0 as usize).unwrap_or(0))
    }

    /// Counts total risk findings in the database
    pub async fn count_findings(&self) -> Result<usize> {
        let result: Vec<(i64,)> = self.db.query("SELECT count() FROM risk_findings").await?.take(0)?;
        Ok(result.first().map(|r| r.0 as usize).unwrap_or(0))
    }

    /// Counts total audit logs in the database
    pub async fn count_audit_logs(&self) -> Result<usize> {
        let result: Vec<(i64,)> = self.db.query("SELECT count() FROM audit_logs").await?.take(0)?;
        Ok(result.first().map(|r| r.0 as usize).unwrap_or(0))
    }

    /// Checks if a skill exists by ID
    pub async fn skill_exists(&self, id: &str) -> Result<bool> {
        let result: Vec<AISkill> = self.db.query("SELECT id FROM ai_skills WHERE id = $id")
            .bind(("id", id))
            .await?
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
        ).await?.take(0)?;
        Ok(skills)
    }

    /// Gets skills sorted by risk count
    pub async fn get_skills_by_risk_count(&self, descending: bool) -> Result<Vec<AISkill>> {
        let order = if descending { "DESC" } else { "ASC" };
        let query = format!("SELECT * FROM ai_skills ORDER BY array::len(risks) {}", order);
        let skills: Vec<AISkill> = self.db.query(query).await?.take(0)?;
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
        ).await?.take(0)?;
        
        for (risk_type, severity, count) in results {
            analysis.entry(risk_type)
                .or_insert_with(HashMap::new)
                .insert(severity, count as usize);
        }
        
        Ok(analysis)
    }

    /// Gets the top N skills with the most risks
    pub async fn get_top_risky_skills(&self, n: usize) -> Result<Vec<AISkill>> {
        let skills: Vec<AISkill> = self.db.query(
            format!("SELECT * FROM ai_skills ORDER BY array::len(risks) DESC LIMIT {}", n)
        ).await?.take(0)?;
        Ok(skills)
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
        ).await?.take(0)?;
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
        .take(0)?;

        let deleted_count = result.first().map(|r| r.0 as usize).unwrap_or(0);
        Ok(deleted_count)
    }
}