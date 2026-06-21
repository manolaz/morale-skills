use crate::models::{RiskFinding, RiskType, Severity};
use crate::risks::base::RiskChecker;
use crate::risks::utils::is_text_file;
use anyhow::Result;
use std::path::Path;
use std::fs;
use async_trait::async_trait;

pub struct ContextPoisoningChecker;

#[async_trait]
impl RiskChecker for ContextPoisoningChecker {
    fn name(&self) -> &'static str {
        "Context Poisoning Checker"
    }

    async fn check(&self, path: &str) -> Result<Vec<RiskFinding>> {
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
                        ("context", Severity::Medium),
                        ("transcript", Severity::Low),
                        ("log", Severity::Low),
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
                    
                    // Look for context overflow vulnerabilities
                    if content.contains("append") && 
                       (content.contains("history") || content.contains("context") || content.contains("conversation")) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::ContextPoisoning,
                            severity: Severity::Medium,
                            description: "Potential context overflow vulnerability".to_string(),
                            evidence: format!("Appending to context/history without limits in {}", file_path.display()),
                            location: Some(file_path.display().to_string()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(findings)
    }

    fn risk_category(&self) -> &'static str {
        "Context Poisoning"
    }
}