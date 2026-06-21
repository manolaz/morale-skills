use crate::database::connection::Database;
use anyhow::Result;
use std::collections::HashMap;

impl Database {
    /// Gets a summary of risks by type
    pub async fn get_risk_summary(&self) -> Result<HashMap<String, usize>> {
        let mut summary = HashMap::new();
        
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
    pub async fn get_severity_summary(&self) -> Result<HashMap<String, usize>> {
        let mut summary = HashMap::new();
        
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
    pub async fn get_status_summary(&self) -> Result<HashMap<String, usize>> {
        let mut summary = HashMap::new();
        
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
}
