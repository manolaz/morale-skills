use crate::models::{RiskFinding, RiskType, Severity};
use anyhow::Result;
use std::path::Path;
use std::fs;

// Helper function to determine if a file is likely a text file
fn is_text_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let text_extensions = ["txt", "py", "js", "ts", "json", "yaml", "yml", "md", "html", "css", "rs", "go", "java", "cpp", "c"];
    
    text_extensions.contains(&ext)
}

pub struct PromptInjectionChecker;

impl PromptInjectionChecker {
    pub async fn check(path: &str) -> Result<Vec<RiskFinding>> {
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
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Look for missing input sanitization
                    if (content.contains("user") || content.contains("input")) && 
                       !(content.contains("sanitize") || content.contains("escape") || content.contains("filter")) {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::PromptInjection,
                            severity: Severity::High,
                            description: "Missing input sanitization for user data".to_string(),
                            evidence: format!("Unsanitized user input in {}", file_path.display()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(findings)
    }
}