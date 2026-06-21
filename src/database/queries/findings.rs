use crate::database::connection::Database;
use crate::models::{AISkill, RiskFinding, RiskType, Severity};
use anyhow::Result;
use serde_json;

impl Database {
    /// Adds a risk finding to a skill
    pub async fn add_risk_finding(&self, skill_id: &str, finding: RiskFinding) -> Result<RiskFinding> {
        // First get the current skill
        let mut skill = self.get_skill(skill_id).await?
            .ok_or_else(|| anyhow::anyhow!("Skill not found: {}", skill_id))?;
        
        // Add the finding to the skill's risks
        skill.risks.push(finding.clone());
        
        // Update the skill in the database
        let _: Option<AISkill> = self.db.update(("ai_skills", skill_id))
            .content(skill)
            .await?;
        
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
            .into_iter()
            .next()
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
}
