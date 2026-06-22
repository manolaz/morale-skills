use crate::models::RiskFinding;
use anyhow::Result;

#[async_trait::async_trait]
pub trait RiskChecker {
    /// Returns the name of the risk checker
    fn name(&self) -> &'static str;
    
    /// Performs the risk check on the given path
    async fn check(&self, path: &str) -> Result<Vec<RiskFinding>>;
    
    /// Returns the category of risk this checker addresses
    fn risk_category(&self) -> &'static str;
}