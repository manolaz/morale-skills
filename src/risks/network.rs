use crate::models::{RiskFinding, RiskType, Severity};
use crate::risks::base::RiskChecker;
use crate::risks::utils::is_text_file;
use anyhow::Result;
use std::path::Path;
use std::fs;
use async_trait::async_trait;

pub struct NetworkChecker;

#[async_trait]
impl RiskChecker for NetworkChecker {
    fn name(&self) -> &'static str {
        "Network Activity Checker"
    }

    async fn check(&self, path: &str) -> Result<Vec<RiskFinding>> {
        let mut findings = Vec::new();
        
        // Walk through all files in the path to look for network-related code
        let dir_entries = fs::read_dir(path)?;
        
        for entry in dir_entries {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_file() && is_text_file(&file_path) {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for network-related patterns
                    let network_patterns = vec![
                        ("http://", Severity::High),
                        ("ftp://", Severity::High),
                        ("https://", Severity::Medium), // Less severe for HTTPS but still worth noting
                        (r"(?i)socket\.", Severity::Medium),
                        (r"(?i)connect\(", Severity::Medium),
                        (r"(?i)request\.", Severity::Low),
                        (r"(?i)urllib", Severity::Low),
                        (r"(?i)requests\.", Severity::Low),
                        (r"(?i)axios", Severity::Low),
                        (r"(?i)fetch\(", Severity::Low),
                        ("hardcoded_url", Severity::Medium),
                        ("external_api", Severity::Medium),
                    ];
                    
                    for (pattern, severity) in &network_patterns {
                        // Skip regex patterns for simple contains check
                        if pattern.starts_with("(?i)") {
                            // This is a regex pattern, we'll handle it differently
                            continue;
                        }
                        
                        if content.contains(pattern) {
                            findings.push(RiskFinding {
                                id: uuid::Uuid::new_v4().to_string(),
                                risk_type: RiskType::Network,
                                severity: severity.clone(),
                                description: format!("Network activity pattern found: {}", pattern),
                                evidence: format!("Pattern '{}' found in file {}", pattern, file_path.display()),
                                location: Some(file_path.display().to_string()),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Check for hardcoded URLs/credentials
                    if content.contains("://") && 
                       (content.contains("token") || content.contains("key") || content.contains("secret")) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::Network,
                            severity: Severity::High,
                            description: "Hardcoded credentials in URL found".to_string(),
                            evidence: format!("Credentials in URL found in {}", file_path.display()),
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
        "Network"
    }
}