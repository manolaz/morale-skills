use crate::models::{RiskFinding, RiskType, Severity};
use anyhow::Result;
use std::fs;
use crate::risks::utils::is_text_file;

pub struct ContextPoisoningChecker;

impl ContextPoisoningChecker {
    pub async fn check(path: &str) -> Result<Vec<RiskFinding>> {
        let mut findings = Vec::new();
        
        // Walk through all files in the path to look for context poisoning risks
        let dir_entries = fs::read_dir(path)?;
        
        for entry in dir_entries {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_file() && is_text_file(&file_path) {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for context poisoning-related patterns
                    let context_patterns = vec![
                        ("history", Severity::Medium),
                        ("conversation", Severity::Medium),
                        ("memory", Severity::Medium),
                        ("cache", Severity::Low),
                        ("session", Severity::Low),
                        ("state", Severity::Medium),
                    ];
                    
                    for (pattern, severity) in &context_patterns {
                        if content.contains(pattern) {
                            findings.push(RiskFinding {
                                id: uuid::Uuid::new_v4().to_string(),
                                risk_type: RiskType::ContextPoisoning,
                                severity: severity.clone(),
                                description: format!("Potential context management found: {}", pattern),
                                evidence: format!("Pattern '{}' found in file {}", pattern, file_path.display()),
                                location: Some(file_path.display().to_string()),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Look for potential data retention without sanitization
                    if content.contains("store") && (content.contains("input") || content.contains("user")) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::ContextPoisoning,
                            severity: Severity::High,
                            description: "Potential unsafe data retention without sanitization".to_string(),
                            evidence: format!("Found 'store' with 'input/user' in {}", file_path.display()),
                            location: Some(file_path.display().to_string()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(findings)
    }
}