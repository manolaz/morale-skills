use crate::models::RiskFinding;
use crate::risks::{
    SupplyChainChecker, NetworkChecker, ContextPoisoningChecker, 
    PromptInjectionChecker, AIAnalyzer, ToolsPermissionsChecker, RiskChecker
};
use anyhow::Result;
use std::sync::Arc;

// A registry of all risk checkers
pub struct RiskCheckerRegistry {
    checkers: Vec<Arc<dyn RiskChecker + Send + Sync>>,
}

impl RiskCheckerRegistry {
    pub fn new() -> Self {
        let checkers: Vec<Arc<dyn RiskChecker + Send + Sync>> = vec![
            Arc::new(SupplyChainChecker {}),
            Arc::new(NetworkChecker {}),
            Arc::new(ContextPoisoningChecker {}),
            Arc::new(PromptInjectionChecker {}),
            Arc::new(AIAnalyzer {}),
            Arc::new(ToolsPermissionsChecker {}), // New checker added
        ];

        Self { checkers }
    }

    pub async fn run_all_checks(&self, path: &str) -> Result<Vec<RiskFinding>> {
        let mut all_findings = Vec::new();

        for checker in &self.checkers {
            let findings = checker.check(path).await?;
            all_findings.extend(findings);
        }

        Ok(all_findings)
    }

    pub fn get_checker_names(&self) -> Vec<&'static str> {
        self.checkers.iter()
            .map(|checker| checker.name())
            .collect()
    }

    pub fn get_checkers_by_category(&self, category: &str) -> Vec<Arc<dyn RiskChecker + Send + Sync>> {
        self.checkers.iter()
            .filter(|checker| checker.risk_category() == category)
            .cloned()
            .collect()
    }
}

// Convenience functions for backward compatibility
pub async fn run_supply_chain_check(path: &str) -> Result<Vec<RiskFinding>> {
    let checker = SupplyChainChecker {};
    checker.check(path).await
}

pub async fn run_network_check(path: &str) -> Result<Vec<RiskFinding>> {
    let checker = NetworkChecker {};
    checker.check(path).await
}

pub async fn run_context_poisoning_check(path: &str) -> Result<Vec<RiskFinding>> {
    let checker = ContextPoisoningChecker {};
    checker.check(path).await
}

pub async fn run_prompt_injection_check(path: &str) -> Result<Vec<RiskFinding>> {
    let checker = PromptInjectionChecker {};
    checker.check(path).await
}

pub async fn run_ai_analysis_check(path: &str) -> Result<Vec<RiskFinding>> {
    let checker = AIAnalyzer {};
    checker.check(path).await
}

pub async fn run_tools_permissions_check(path: &str) -> Result<Vec<RiskFinding>> {
    let checker = ToolsPermissionsChecker {}; // New function for the new checker
    checker.check(path).await
}