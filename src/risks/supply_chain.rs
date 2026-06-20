use crate::models::{RiskFinding, RiskType, Severity};
use anyhow::Result;
use std::path::Path;
use std::fs;
use regex::Regex;

pub struct SupplyChainChecker;

impl SupplyChainChecker {
    pub async fn check(path: &str) -> Result<Vec<RiskFinding>> {
        let mut findings = Vec::new();
        
        // Analyze the skill path for supply chain risks
        if path.contains("untrusted") || path.contains("unknown") {
            findings.push(RiskFinding {
                id: uuid::Uuid::new_v4().to_string(),
                risk_type: RiskType::SupplyChain,
                severity: Severity::High,
                description: "Skill originates from untrusted source".to_string(),
                evidence: format!("Path contains suspicious indicators: {}", path),
                timestamp: chrono::Utc::now(),
            });
        }
        
        // Check for common dependency files that might indicate supply chain risks
        let dependency_files = [
            "requirements.txt",
            "package-lock.json", 
            "yarn.lock",
            "Gemfile.lock",
            "Cargo.lock",
            "go.mod",
            "poetry.lock"
        ];
        
        for file in &dependency_files {
            let full_path = Path::new(path).join(file);
            if full_path.exists() {
                // Analyze the dependency file for potentially risky packages
                if let Ok(content) = fs::read_to_string(&full_path) {
                    let risky_packages = vec![
                        "eval\\(", "exec\\(", "importlib", "subprocess", "__import__", 
                        "os.system", "os.popen", "subprocess.run", "subprocess.Popen"
                    ];
                    
                    if let Some(matched) = re.find(content) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::SupplyChain,
                            severity: Severity::Medium,
                            description: format!("Potentially risky package/function found in {}: {}", file, matched.as_str()),
                            evidence: format!("Found '{}' in {}", matched.as_str(), full_path.display()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(findings)
    }
}