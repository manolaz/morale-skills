use anyhow::Result;
use morale::database::Database;
use morale::models::{AISkill, RiskFinding, RiskType, Severity};

#[tokio::test]
async fn test_database_operations() -> Result<()> {
    let db = Database::new().await?;
    
    let skill = AISkill::new(
        "test_skill".to_string(),
        Some("A test skill for integration testing".to_string()),
        "/tmp/test_skill".to_string(),
    );
    
    let saved_skill = db.save_skill(skill).await?;
    assert_eq!(saved_skill.name, "test_skill");
    
    Ok(())
}

#[tokio::test]
async fn test_risk_finding_creation() -> Result<()> {
    let risk = RiskFinding::new(
        RiskType::SupplyChain,
        Severity::High,
        "Test risk finding".to_string(),
        "Evidence for test risk".to_string(),
        None,
    );
    
    assert_eq!(risk.risk_type.to_string(), "Supply Chain");
    assert_eq!(risk.severity.to_string(), "High");
    
    Ok(())
}