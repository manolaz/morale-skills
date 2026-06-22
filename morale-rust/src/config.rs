use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub audit: AuditConfig,
    pub report: ReportConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub storage_engine: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditConfig {
    pub verbose: bool,
    pub risk_thresholds: std::collections::HashMap<String, u8>,
    pub enabled_checks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReportConfig {
    pub output_format: String,
    pub output_directory: String,
    pub include_evidence: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub max_file_size_mb: u64,
    pub allowed_extensions: Vec<String>,
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn default() -> Self {
        Config {
            database: DatabaseConfig {
                storage_engine: "mem".to_string(),
                namespace: "morale".to_string(),
                database: "skills".to_string(),
            },
            audit: AuditConfig {
                verbose: false,
                risk_thresholds: [
                    ("low".to_string(), 0),
                    ("medium".to_string(), 1),
                    ("high".to_string(), 2),
                    ("critical".to_string(), 3),
                ].iter().cloned().collect(),
                enabled_checks: vec![
                    "supply_chain".to_string(),
                    "network".to_string(),
                    "context_poisoning".to_string(),
                    "prompt_injection".to_string(),
                ],
            },
            report: ReportConfig {
                output_format: "json".to_string(),
                output_directory: "./reports".to_string(),
                include_evidence: true,
            },
            security: SecurityConfig {
                max_file_size_mb: 100,
                allowed_extensions: vec![
                    ".py".to_string(),
                    ".js".to_string(),
                    ".ts".to_string(),
                    ".json".to_string(),
                    ".yaml".to_string(),
                    ".yml".to_string(),
                    ".txt".to_string(),
                    ".md".to_string(),
                ],
            },
        }
    }
    
    pub fn is_check_enabled(&self, check_name: &str) -> bool {
        self.audit.enabled_checks.iter().any(|check| check == check_name)
    }
}

// Add the config module to the lib.rs file