use crate::models::{RiskFinding, RiskType, Severity};
use anyhow::Result;
use std::path::Path;
use std::fs;
use crate::risks::utils::is_text_file;

pub struct NetworkChecker;

impl NetworkChecker {
    pub async fn check(path: &str) -> Result<Vec<RiskFinding>> {
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
                        (r"(?i)socket\.", Severity::Medium),
                        (r"(?i)connect\(", Severity::Medium),
                        (r"(?i)request\.", Severity::Low),
                        (r"(?i)urllib", Severity::Low),
                        (r"(?i)requests\.", Severity::Low),
                        (r"(?i)axios", Severity::Low),
                        (r"(?i)fetch\(", Severity::Low),
                    ];
                    
                    for (pattern, severity) in &network_patterns {
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
                }
            }
        }
        
        Ok(findings)
    }
}