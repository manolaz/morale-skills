use crate::database::connection::Database;
use crate::models::{AISkill, RiskFinding, AuditLog};
use anyhow::Result;
use serde_json;

impl Database {
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
                            let _: Option<AISkill> = self.db.update(("ai_skills", &skill.id))
                                .content(skill)
                                .await?;
                        },
                        None => {
                            let _: Vec<AISkill> = self.db.create("ai_skills")
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
                    let _: Vec<RiskFinding> = self.db.create("risk_findings")
                        .content(finding)
                        .await?;
                }
            }
        }

        if let Some(logs_array) = data.get("logs").and_then(|v| v.as_array()) {
            for log_value in logs_array {
                if let Ok(log) = serde_json::from_value::<AuditLog>(log_value.clone()) {
                    let _: Vec<AuditLog> = self.db.create("audit_logs")
                        .content(log)
                        .await?;
                }
            }
        }

        Ok(())
    }
}