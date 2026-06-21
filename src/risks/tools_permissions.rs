use crate::models::{RiskFinding, RiskType, Severity};
use crate::risks::base::RiskChecker;
use anyhow::Result;
use std::path::Path;
use std::fs;
use async_trait::async_trait;

pub struct ToolsPermissionsChecker;

#[async_trait]
impl RiskChecker for ToolsPermissionsChecker {
    fn name(&self) -> &'static str {
        "Tools Permissions Checker"
    }

    async fn check(&self, path: &str) -> Result<Vec<RiskFinding>> {
        let mut findings = Vec::new();
        
        // Walk through all files in the path to look for tools permissions risks
        let dir_entries = fs::read_dir(path)?;
        
        for entry in dir_entries {
            let entry = entry?;
            let file_path = entry.path();
            
            if file_path.is_file() && crate::risks::utils::is_text_file(&file_path) {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    // Look for tools permissions-related patterns
                    let permission_patterns = vec![
                        ("permission", Severity::Medium),
                        ("access_token", Severity::High),
                        ("api_key", Severity::High),
                        ("secret", Severity::High),
                        ("credentials", Severity::High),
                        ("authorization", Severity::Medium),
                        ("auth", Severity::Medium),
                        ("oauth", Severity::Medium),
                        ("bearer", Severity::Medium),
                        ("admin", Severity::Low),
                        ("root", Severity::Low),
                        ("sudo", Severity::Low),
                        ("privileged", Severity::Medium),
                        ("execute", Severity::Medium),
                        ("shell", Severity::High),
                        ("command", Severity::Medium),
                        ("system", Severity::Medium),
                    ];
                    
                    for (pattern, severity) in &permission_patterns {
                        if content.contains(pattern) {
                            findings.push(RiskFinding {
                                id: uuid::Uuid::new_v4().to_string(),
                                risk_type: RiskType::AIBased, // Using AIBased for AI-specific risks
                                severity: severity.clone(),
                                description: format!("Potential tools permissions issue found: {}", pattern),
                                evidence: format!("Pattern '{}' found in file {}", pattern, file_path.display()),
                                location: Some(file_path.display().to_string()),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Look for overly permissive tool configurations
                    if content.contains("permissions") && content.contains("*") {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::AIBased,
                            severity: Severity::High,
                            description: "Overly permissive tool configuration found".to_string(),
                            evidence: format!("Found wildcard permissions in {}", file_path.display()),
                            location: Some(file_path.display().to_string()),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                    
                    // Look for tools that could be dangerous if misused
                    let dangerous_tools = vec![
                        "eval", "exec", "compile", "__import__", 
                        "open", "write", "remove", "delete", 
                        "subprocess", "os.system", "os.popen", 
                        "shell_exec", "system_exec", "popen"
                    ];
                    
                    for tool in &dangerous_tools {
                        if content.contains(tool) {
                            findings.push(RiskFinding {
                                id: uuid::Uuid::new_v4().to_string(),
                                risk_type: RiskType::AIBased,
                                severity: Severity::High,
                                description: format!("Potentially dangerous tool found: {}", tool),
                                evidence: format!("Dangerous tool '{}' found in {}", tool, file_path.display()),
                                location: Some(file_path.display().to_string()),
                                timestamp: chrono::Utc::now(),
                            });
                        }
                    }
                    
                    // Look for tools permissions that might allow excessive access
                    if content.contains("allow") && content.contains("all") {
                        findings.push(RiskFinding {
                            id: uuid::Uuid::new_v4().to_string(),
                            risk_type: RiskType::AIBased,
                            severity: Severity::Medium,
                            description: "Excessive permissions granted".to_string(),
                            evidence: format!("Found 'allow all' pattern in {}", file_path.display()),
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
        "Tools Permissions"
    }
}