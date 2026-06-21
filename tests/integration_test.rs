use anyhow::Result;
use morale::database::Database;
use morale::models::{AISkill, RiskFinding, RiskType, Severity};

#[tokio::test]
async fn test_database_operations() -> Result<()> {
    let db = Database::new().await?;
    
    let skill = AISkill {
        id: uuid::Uuid::new_v4().to_string(),
        name: "test_skill".to_string(),
        description: Some("A test skill for integration testing".to_string()),
        file_path: "/tmp/test_skill".to_string(),
        created_at: Some(chrono::Utc::now()),
        risks: vec![],
        status: "pending".to_string(),
    };
    
    let saved_skill = db.save_skill(skill).await?;
    assert_eq!(saved_skill.name, "test_skill");
    
    Ok(())
}

#[tokio::test]
async fn test_risk_finding_creation() -> Result<()> {
    let risk = RiskFinding {
        id: uuid::Uuid::new_v4().to_string(),
        risk_type: RiskType::SupplyChain,
        severity: Severity::High,
        description: "Test risk finding".to_string(),
        evidence: "Evidence for test risk".to_string(),
        location: None,
        timestamp: chrono::Utc::now(),
    };
    
    assert_eq!(risk.risk_type.to_string(), "Supply Chain");
    assert_eq!(risk.severity.to_string(), "High");
    
    Ok(())
}