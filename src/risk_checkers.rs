use crate::models::{RiskFinding, RiskType, Severity};
use anyhow::Result;
use std::path::Path;
use std::fs;
use tokio;

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
                    
                    for pkg in &risky_packages {
                        if content.contains(pkg) {
                            findings.push(RiskFinding {
                                id: uuid::Uuid::new_v4().to_string(),
                                risk_type: RiskType::SupplyChain,
                                severity: Severity::Medium,
                                description: format!("Potentially risky package/function found in {}: {}", file, pkg),
                                evidence: format!("Found '{}' in {}", pkg, full_path.display()),
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
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(findings)
    }
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

// Re-export risk checkers from the modular structure
pub use crate::risks::{
    SupplyChainChecker, 
    NetworkChecker, 
    ContextPoisoningChecker, 
    PromptInjectionChecker,
    AIAnalyzer
};

// Keep the is_text_file helper function here since multiple modules used it
use std::path::Path;

// Helper function to determine if a file is likely a text file
pub fn is_text_file(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let text_extensions = ["txt", "py", "js", "ts", "json", "yaml", "yml", "md", "html", "css", "rs", "go", "java", "cpp", "c"];
    
    text_extensions.contains(&ext)
}