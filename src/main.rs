use anyhow::Result;
use clap::Parser;
use tracing::{info, error, warn, debug, Level};
use std::path::Path;
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};

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

    /// Run in lightweight mode (skip database operations)
    #[arg(short, long)]
    lightweight: bool,
}

struct MoraleAuditor {
    db: Option<Database>,
}

impl MoraleAuditor {
    async fn new(lightweight: bool) -> Result<Self> {
        let db = if lightweight {
            None
        } else {
            Some(Database::new().await?)
        };
        
        Ok(MoraleAuditor { db })
    }
    
    async fn audit_skill(&self, path: &str, use_ai: bool, ai_service: &str, ai_api_key: Option<&str>, ai_endpoint: Option<&str>) -> Result<AISkill> {
        info!("Starting audit for skill at path: {}", path);
        
        // Validate input path exists
        let path_obj = std::path::Path::new(path);
        if !path_obj.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path));
        }
        
        if !path_obj.is_dir() && !path.ends_with(".zip") {
            return Err(anyhow::anyhow!("Path must be a directory or a ZIP file: {}", path));
        }
        
        let skill_name = Path::new(path)
            .file_stem()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Create a progress bar to show audit progress
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap());
        pb.set_message("Initializing risk checkers...");
        
        // Create a registry of risk checkers and run all checks
        let registry = RiskCheckerRegistry::new();
        pb.set_message("Running risk checks...");
        info!("Running risk checks on skill...");
        
        // Run checks with progress tracking
        let mut risks = registry.run_all_checks(path).await?;
        pb.finish_with_message("Risk checks completed");
        info!("Completed risk checks. Found {} risks", risks.len());
        
        // Perform AI-based analysis if enabled
        if use_ai {
            if let Some(api_key) = ai_api_key {
                info!("Performing AI-based analysis using {}", ai_service);
                
                // Get all text files for AI analysis
                let text_files = self.get_text_files(path).await?;
                info!("Processing {} text files for AI analysis", text_files.len());
                
                if !text_files.is_empty() {
                    // Show progress for AI analysis
                    let ai_pb = ProgressBar::new(text_files.len() as u64);
                    ai_pb.set_style(ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                        .unwrap());
                    ai_pb.set_message("Analyzing files with AI...");
                    
                    // Process files in parallel for better performance
                    let mut ai_findings = Vec::new();
                    for (idx, file_content) in text_files.into_iter().enumerate() {
                        match morale::risks::AIAnalyzer::analyze_code_with_ai(
                            &file_content,
                            ai_service,
                            api_key,
                            ai_endpoint,
                        ).await {
                            Ok(findings) => {
                                debug!("AI analysis found {} risks in file {}", findings.len(), idx + 1);
                                ai_findings.extend(findings);
                            },
                            Err(e) => {
                                warn!("AI analysis failed for a file: {}", e);
                                // Continue with other files instead of failing completely
                            }
                        }
                        ai_pb.inc(1);
                    }
                    ai_pb.finish_with_message("AI analysis completed");
                    
                    let ai_count = ai_findings.len();
                    risks.extend(ai_findings);
                    info!("Added {} AI-detected risks", ai_count);
                } else {
                    info!("No text files found for AI analysis");
                }
            } else {
                error!("AI analysis requested but no API key provided");
            }
        }
        
        // Create AISkill with proper initialization matching models.rs
        let mut skill = AISkill::new(
            skill_name,
            Some(format!("AI skill from path: {}", path)),
            path.to_string(),
        );
        
        // Update the risks on the skill
        skill.risks = risks;
        
        // Save the audit result to the database if available
        let created_skill = if let Some(ref db) = self.db {
            info!("Saving audit results to database...");
            db.save_skill(skill).await?
        } else {
            info!("Lightweight mode: skipping database save.");
            skill
        };
        
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
                                    // Limit content size and skip empty files
                                    if content.len() > 0 && content.len() < 10000 {
                                        contents.push(content);
                                    } else if content.len() >= 10000 {
                                        debug!("Skipping large file ({} chars): {:?}", content.len(), entry_path);
                                    }
                                } else {
                                    warn!("Could not read file: {:?}", entry_path);
                                }
                            }
                        }
                    }
                } else {
                    warn!("Could not read directory: {:?}", current_path);
                }
            }
        }
        
        Ok(contents)
    }
    
    async fn get_audit_report(&self, skill_id: &str) -> Result<Option<AISkill>> {
        if let Some(ref db) = self.db {
            db.get_skill(skill_id).await
        } else {
            Err(anyhow::anyhow!("Database is not available in lightweight mode"))
        }
    }
    
    async fn get_all_audits(&self) -> Result<Vec<AISkill>> {
        if let Some(ref db) = self.db {
            db.get_all_skills().await
        } else {
            Err(anyhow::anyhow!("Database is not available in lightweight mode"))
        }
    }
    
    async fn get_skills_by_risk_type(&self, risk_type: &RiskType) -> Result<Vec<AISkill>> {
        if let Some(ref db) = self.db {
            db.get_skills_by_risk_type(risk_type).await
        } else {
            Err(anyhow::anyhow!("Database is not available in lightweight mode"))
        }
    }
    
    async fn get_skills_by_severity(&self, severity: &Severity) -> Result<Vec<AISkill>> {
        if let Some(ref db) = self.db {
            db.get_skills_by_severity(severity).await
        } else {
            Err(anyhow::anyhow!("Database is not available in lightweight mode"))
        }
    }
    
    async fn get_risk_summary(&self) -> Result<HashMap<String, usize>> {
        if let Some(ref db) = self.db {
            db.get_risk_summary().await
        } else {
            Err(anyhow::anyhow!("Database is not available in lightweight mode"))
        }
    }
    
    async fn get_severity_summary(&self) -> Result<HashMap<String, usize>> {
        if let Some(ref db) = self.db {
            db.get_severity_summary().await
        } else {
            Err(anyhow::anyhow!("Database is not available in lightweight mode"))
        }
    }
}

fn print_report(skill: &AISkill, json: bool) {
    if json {
        match serde_json::to_string_pretty(&skill) {
            Ok(json_output) => println!("{}", json_output),
            Err(e) => {
                error!("Failed to serialize report to JSON: {}", e);
                // Fallback to plain text output
                print_report_plain(skill);
            }
        }
    } else {
        print_report_plain(skill);
    }
}

fn print_report_plain(skill: &AISkill) {
    println!("Audit Report");
    println!("============");
    println!("Skill: {}", skill.name);
    println!("Path: {}", skill.file_path);
    println!("Created: {}", skill.created_at.unwrap_or_else(|| chrono::Utc::now()));
    println!("Status: {}", skill.status);
    println!("Risks found: {}", skill.risks.len());
    println!();
    
    if !skill.risks.is_empty() {
        // Group risks by severity for better presentation
        let mut risks_by_severity: HashMap<String, Vec<&RiskFinding>> = HashMap::new();
        for risk in &skill.risks {
            risks_by_severity.entry(risk.severity.to_string()).or_insert_with(Vec::new).push(risk);
        }
        
        // Print risks sorted by severity (critical first)
        let severity_order = [Severity::Critical, Severity::High, Severity::Medium, Severity::Low];
        for severity in &severity_order {
            let severity_str = severity.to_string();
            if let Some(risks) = risks_by_severity.get(&severity_str) {
                println!("\n--- {} RISKS ({}) ---", severity_str.to_uppercase(), risks.len());
                for (i, risk) in risks.iter().enumerate() {
                    println!("{}. {} ({})", i + 1, risk.description, risk.severity.to_string());
                    println!("   Type: {}", risk.risk_type.to_string());
                    println!("   Evidence: {}", risk.evidence);
                    if let Some(location) = &risk.location {
                        println!("   Location: {}", location);
                    }
                    println!();
                }
            }
        }
    } else {
        println!("No risks detected in the skill.");
    }
}

fn print_summary(audits: &[AISkill]) {
    println!("Audit Summary");
    println!("=============");
    println!("Total skills audited: {}", audits.len());
    
    let mut total_risks = 0;
    let mut risks_by_severity: HashMap<String, usize> = HashMap::new();
    
    for skill in audits {
        total_risks += skill.risks.len();
        for risk in &skill.risks {
            *risks_by_severity.entry(risk.severity.to_string()).or_insert(0) += 1;
        }
    }
    
    println!("Total risks found: {}", total_risks);
    
    if !audits.is_empty() {
        let avg_risks = total_risks as f64 / audits.len() as f64;
        println!("Average risks per skill: {:.2}", avg_risks);
    }
    
    // Print breakdown by severity
    if !risks_by_severity.is_empty() {
        println!("\nRisk Breakdown by Severity:");
        for (severity, count) in &risks_by_severity {
            println!("  {}: {}", severity, count);
        }
    }
    
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging based on verbosity
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(Level::WARN)
            .init();
    }
    
    debug!("Initializing MoraleAuditor...");
    let auditor = MoraleAuditor::new(args.lightweight).await?;
    debug!("MoraleAuditor initialized successfully");
    
    // Handle different command modes
    if args.list {
        debug!("Fetching all audits from database...");
        let audits = auditor.get_all_audits().await?;
        println!("All Audits:");
        for skill in &audits {
            println!("- {} ({} risks)", skill.name, skill.risks.len());
        }
        return Ok(());
    }
    
    if args.stats {
        debug!("Fetching risk statistics...");
        let risk_summary = auditor.get_risk_summary().await?;
        let severity_summary = auditor.get_severity_summary().await?;
        
        println!("Risk Statistics:");
        println!("================");
        println!("Risk Types:");
        if risk_summary.is_empty() {
            println!("  No risks found in the database");
        } else {
            for (risk_type, count) in &risk_summary {
                println!("  {}: {}", risk_type, count);
            }
        }
        
        println!("\nSeverity Levels:");
        if severity_summary.is_empty() {
            println!("  No risks found in the database");
        } else {
            for (severity, count) in &severity_summary {
                println!("  {}: {}", severity, count);
            }
        }
        return Ok(());
    }
    
    if args.summary {
        debug!("Generating audit summary...");
        let audits = auditor.get_all_audits().await?;
        print_summary(&audits);
        return Ok(());
    }
    
    // Perform the audit with AI integration if enabled
    debug!("Starting audit for: {}", args.target_path);
    match auditor.audit_skill(
        &args.target_path,
        args.enable_ai,
        &args.ai_service,
        args.ai_api_key.as_deref(),
        args.ai_endpoint.as_deref(),
    ).await {
        Ok(skill) => {
            debug!("Audit completed successfully for: {}", skill.name);
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
        let result = MoraleAuditor::new(false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_morale_auditor_creation_lightweight() {
        let result = MoraleAuditor::new(true).await;
        assert!(result.is_ok());
        let auditor = result.unwrap();
        assert!(auditor.db.is_none());
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