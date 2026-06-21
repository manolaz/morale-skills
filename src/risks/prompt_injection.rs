use crate::models::{RiskFinding, RiskType, Severity};
use crate::risks::base::RiskChecker;
use crate::risks::utils::is_text_file;
use anyhow::Result;
use std::fs;
use async_trait::async_trait;

pub struct PromptInjectionChecker;

#[async_trait]
impl RiskChecker for PromptInjectionChecker {
    fn name(&self) -> &'static str {
        "Prompt Injection Checker"
    }

    async fn check(&self, path: &str) -> Result<Vec<RiskFinding>> {
        let mut findings = Vec::new();
        
        // Walk through all files in the path to look for prompt injection risks
        let dir_entries = fs::read_dir(path)?;
        
        for entry in dir_entries {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_file() && is_text_file(&file_path) {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for prompt injection-related patterns
                    let injection_patterns = vec![
                        ("f\"", Severity::High), // f-strings in Python
                        ("f'", Severity::High),
                        ("template", Severity::Medium),
                        ("format", Severity::Medium),
                        ("concat", Severity::Medium),
                        ("replace", Severity::Low),
                        ("input", Severity::High), // Direct user input handling
                        ("prompt", Severity::Medium),
                        ("message", Severity::Medium),
                        ("query", Severity::Medium),
                    ];
                    
                    for (pattern, severity) in &injection_patterns {
                        if content.contains(pattern) && 
                           (content.contains("prompt") || content.contains("message") || content.contains("query")) {
                            findings.push(RiskFinding {
                                id: uuid::Uuid::new_v4().to_string(),
                                risk_type: RiskType::PromptInjection,
                                severity: severity.clone(),
                                description: format!("Potential prompt injection vulnerability: {}", pattern),
                                evidence: format!("Pattern '{}' found near prompt-related code in {}", pattern, file_path.display()),
                                location: Some(file_path.display().to_string()),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Look for missing input sanitization
                    if (content.contains("user") || content.contains("input")) && 
                       !(content.contains("sanitize") || content.contains("escape") || content.contains("filter") || content.contains("validate")) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::PromptInjection,
                            severity: Severity::High,
                            description: "Missing input sanitization for user data".to_string(),
                            evidence: format!("Unsanitized user input in {}", file_path.display()),
                            location: Some(file_path.display().to_string()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                    
                    // Look for direct concatenation of user input in prompts
                    if content.contains("+") && 
                       (content.contains("prompt") || content.contains("message")) && 
                       (content.contains("user") || content.contains("input")) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::PromptInjection,
                            severity: Severity::High,
                            description: "Direct concatenation of user input in prompts".to_string(),
                            evidence: format!("Direct concatenation found in {}", file_path.display()),
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
        "Prompt Injection"
    }
}