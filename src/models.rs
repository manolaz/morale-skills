use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents an AI Skill being audited
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AISkill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub file_path: String,
    pub created_at: Option<DateTime<Utc>>,
    pub risks: Vec<RiskFinding>,
    pub status: String,
}

impl AISkill {
    pub fn new(name: String, description: Option<String>, file_path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            file_path,
            created_at: Some(Utc::now()),
            risks: Vec::new(),
            status: "pending".to_string(),
        }
    }
}

/// Represents a risk finding during the audit
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RiskFinding {
    pub id: String,
    pub risk_type: RiskType,
    pub severity: Severity,
    pub description: String,
    pub evidence: String,
    pub location: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl RiskFinding {
    pub fn new(risk_type: RiskType, severity: Severity, description: String, evidence: String, location: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            risk_type,
            severity,
            description,
            evidence,
            location,
            timestamp: Utc::now(),
        }
    }
}

/// Represents an audit log entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditLog {
    pub id: String,
    pub skill_id: String,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(skill_id: String, action: String, details: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            skill_id,
            action,
            details,
            timestamp: Utc::now(),
        }
    }
}

/// Types of risks that can be detected
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum RiskType {
    SupplyChain,
    Network,
    ContextPoisoning,
    PromptInjection,
    AIBased,
    #[serde(rename = "other")]
    Other(String),
}

/// Severity levels for risk findings
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Severity {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

impl ToString for RiskType {
    fn to_string(&self) -> String {
        match self {
            RiskType::SupplyChain => "Supply Chain".to_string(),
            RiskType::Network => "Network".to_string(),
            RiskType::ContextPoisoning => "Context Poisoning".to_string(),
            RiskType::PromptInjection => "Prompt Injection".to_string(),
            RiskType::AIBased => "AI-Based Analysis".to_string(),
            RiskType::Other(s) => s.clone(),
        }
    }
}

impl ToString for Severity {
    fn to_string(&self) -> String {
        match self {
            Severity::Low => "Low".to_string(),
            Severity::Medium => "Medium".to_string(),
            Severity::High => "High".to_string(),
            Severity::Critical => "Critical".to_string(),
        }
    }
}