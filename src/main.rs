use anyhow::Result;
use clap::Parser;
use tracing::{info, error, Level};
use std::path::Path;
use std::collections::HashMap;

use morale::{database::Database, models::{AISkill, RiskFinding, RiskType, Severity}, risk_checkers::RiskCheckerRegistry};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the AI skill folder or ZIP file to audit
    #[arg(value_parser)]
    target_path: String,
    
    /// Run in verbose mode
    #[arg(short, long)]
    verbose: bool,
    
    /// Output report in JSON format
    #[arg(long)]
    json: bool,
    
    /// Generate a summary report only
    #[arg(long)]
    summary: bool,
    
    /// List all audits in the database
    #[arg(long)]
    list: bool,
    
    /// Show risk statistics
    #[arg(long)]
    stats: bool,
    
    /// Enable AI-powered analysis (requires API key)
    #[arg(long)]
    enable_ai: bool,
    
    /// AI service to use (claude, codex, gemini, qwen)
    #[arg(long, default_value = "claude")]
    ai_service: String,
    
    /// AI API key for the selected service
    #[arg(long)]
    ai_api_key: Option<String>,
    
    /// Custom endpoint for AI service (required for Qwen)
    #[arg(long)]
    ai_endpoint: Option<String>,
}

struct MoraleAuditor {
    db: Database,
}

impl MoraleAuditor {
    async fn new() -> Result<Self> {
        let db = Database::new().await?;
        
        Ok(MoraleAuditor { db })
    }
    
    async fn audit_skill(&self, path: &str, use_ai: bool, ai_service: &str, ai_api_key: Option<&str>, ai_endpoint: Option<&str>) -> Result<AISkill> {
        info!("Starting audit for skill at path: {}", path);
        
        let skill_name = Path::new(path)
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Create a registry of risk checkers and run all checks
        let registry = RiskCheckerRegistry::new();
        let mut risks = registry.run_all_checks(path).await?;
        
        // Perform AI-based analysis if enabled
        if use_ai {
            if let Some(api_key) = ai_api_key {
                info!("Performing AI-based analysis using {}", ai_service);
                
                // Get all text files for AI analysis
                let text_files = self.get_text_files(path).await?;
                
                for file_content in text_files {
                    match morale::risks::AIAnalyzer::analyze_code_with_ai(
                        &file_content,
                        ai_service,
                        api_key,
                        ai_endpoint,
                    ).await {
                        Ok(ai_findings) => risks.extend(ai_findings),
                        Err(e) => error!("AI analysis failed: {}", e),
                    }
                }
            } else {
                error!("AI analysis requested but no API key provided");
            }
        }
        
        let skill = AISkill {
            id: None,
            name: skill_name,
            description: Some(format!("AI skill from path: {}", path)),
            file_path: path.to_string(),
            created_at: chrono::Utc::now(),
            risks,
        };
        
        // Save the audit result to the database
        let created_skill = self.db.save_skill(skill).await?;
        
        info!("Audit completed for skill: {}", created_skill.name);
        Ok(created_skill)
    }
    
    // Helper function to get all text file contents from a path
    async fn get_text_files(&self, path: &str) -> Result<Vec<String>> {
        use std::fs;
        use std::path::PathBuf;
        
        let mut contents = Vec::new();
        let mut stack = vec![PathBuf::from(path)];
        
        while let Some(current_path) = stack.pop() {
            if current_path.is_dir() {
                if let Ok(entries) = fs::read_dir(&current_path) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();
                            if entry_path.is_dir() {
                                stack.push(entry_path);
                            } else if morale::risks::utils::is_text_file(&entry_path) {
                                if let Ok(content) = fs::read_to_string(&entry_path) {
                                    if content.len() > 0 && content.len() < 10000 { // Limit content size
                                        contents.push(content);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(contents)
    }
    
    async fn get_audit_report(&self, skill_id: &str) -> Result<Option<AISkill>> {
        self.db.get_skill(skill_id).await
    }
    
    async fn get_all_audits(&self) -> Result<Vec<AISkill>> {
        self.db.get_all_skills().await
    }
    
    async fn get_skills_by_risk_type(&self, risk_type: &RiskType) -> Result<Vec<AISkill>> {
        self.db.get_skills_by_risk_type(risk_type).await
    }
    
    async fn get_skills_by_severity(&self, severity: &Severity) -> Result<Vec<AISkill>> {
        self.db.get_skills_by_severity(severity).await
    }
    
    async fn get_risk_summary(&self) -> Result<HashMap<String, usize>> {
        self.db.get_risk_summary().await
    }
    
    async fn get_severity_summary(&self) -> Result<HashMap<String, usize>> {
        self.db.get_severity_summary().await
    }
}

fn print_report(skill: &AISkill, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(&skill).unwrap());
    } else {
        println!("Audit Report");
        println!("============");
        println!("Skill: {}", skill.name);
        println!("Path: {}", skill.file_path);
        println!("Created: {}", skill.created_at);
        println!("Risks found: {}", skill.risks.len());
        println!();
        
        if !skill.risks.is_empty() {
            for (i, risk) in skill.risks.iter().enumerate() {
                println!("{}. {} ({})", i + 1, risk.description, risk.severity.to_string());
                println!("   Type: {}", risk.risk_type.to_string());
                println!("   Evidence: {}", risk.evidence);
                println!();
            }
        } else {
            println!("No risks detected in the skill.");
        }
    }
}

fn print_summary(audits: &[AISkill]) {
    println!("Audit Summary");
    println!("=============");
    println!("Total skills audited: {}", audits.len());
    
    let mut total_risks = 0;
    for skill in audits {
        total_risks += skill.risks.len();
    }
    println!("Total risks found: {}", total_risks);
    
    if !audits.is_empty() {
        let avg_risks = total_risks as f64 / audits.len() as f64;
        println!("Average risks per skill: {:.2}", avg_risks);
    }
    
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging based on verbosity
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(Level::ERROR)
            .init();
    }
    
    let auditor = MoraleAuditor::new().await?;
    
    // Handle different command modes
    if args.list {
        let audits = auditor.get_all_audits().await?;
        println!("All Audits:");
        for skill in &audits {
            println!("- {} ({} risks)", skill.name, skill.risks.len());
        }
        return Ok(());
    }
    
    if args.stats {
        let risk_summary = auditor.get_risk_summary().await?;
        let severity_summary = auditor.get_severity_summary().await?;
        
        println!("Risk Statistics:");
        println!("================");
        println!("Risk Types:");
        for (risk_type, count) in &risk_summary {
            println!("  {}: {}", risk_type, count);
        }
        
        println("\nSeverity Levels:");
        for (severity, count) in &severity_summary {
            println!("  {}: {}", severity, count);
        }
        return Ok(());
    }
    
    if args.summary {
        let audits = auditor.get_all_audits().await?;
        print_summary(&audits);
        return Ok(());
    }
    
    // Perform the audit with AI integration if enabled
    match auditor.audit_skill(
        &args.target_path,
        args.enable_ai,
        &args.ai_service,
        args.ai_api_key.as_deref(),
        args.ai_endpoint.as_deref(),
    ).await {
        Ok(skill) => {
            if args.summary {
                print_summary(&[skill]);
            } else {
                print_report(&skill, args.json);
            }
        }
        Err(e) => {
            error!("Error during audit: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_morale_auditor_creation() {
        let result = MoraleAuditor::new().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_risk_types_to_string() {
        assert_eq!(RiskType::SupplyChain.to_string(), "Supply Chain");
        assert_eq!(RiskType::Network.to_string(), "Network");
        assert_eq!(RiskType::ContextPoisoning.to_string(), "Context Poisoning");
        assert_eq!(RiskType::PromptInjection.to_string(), "Prompt Injection");
        assert_eq!(RiskType::AIBased.to_string(), "AI-Based Analysis");
    }
    
    #[tokio::test]
    async fn test_severity_to_string() {
        assert_eq!(Severity::Low.to_string(), "Low");
        assert_eq!(Severity::Medium.to_string(), "Medium");
        assert_eq!(Severity::High.to_string(), "High");
        assert_eq!(Severity::Critical.to_string(), "Critical");
    }
}