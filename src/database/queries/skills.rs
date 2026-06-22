use crate::database::connection::Database;
use crate::models::{AISkill, RiskType, Severity};
use anyhow::Result;
use chrono;
use serde_json;

impl Database {
    /// Saves a skill to the database
    pub async fn save_skill(&self, mut skill: AISkill) -> Result<AISkill> {
        // Ensure created_at is set
        if skill.created_at.is_none() {
            skill.created_at = Some(chrono::Utc::now());
        }
        
        let created_skill: AISkill = self.db.create("ai_skills")
            .content(skill)
            .await?
            .into_iter()
            .next()
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
}
