use crate::models::{RiskFinding, RiskType, Severity};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
struct APIRequest {
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct APIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    text: String,
}

pub struct AIAnalyzer;

impl AIAnalyzer {
    /// Analyze code using Anthropic's Claude API
    pub async fn analyze_with_claude(code_snippet: &str, api_key: &str) -> Result<Vec<RiskFinding>> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "Analyze the following code snippet for potential security vulnerabilities and risks. Focus on supply chain, network, context poisoning, and prompt injection vulnerabilities. Return your response in JSON format with fields: risk_type, severity (High/Medium/Low), description, and evidence.\n\nCode:\n{}",
            code_snippet
        );

        let request_body = serde_json::json!({
            "model": "claude-3-opus-20240229",
            "prompt": prompt,
            "max_tokens_to_sample": 1000,
            "temperature": 0.3
        });

        let response = client
            .post("https://api.anthropic.com/v1/complete")
            .header("Content-Type", "application/json")
            .header("x-api-key", api_key)
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            parse_ai_response(response_text, RiskType::AIBased).await
        } else {
            Ok(vec![])
        }
    }

    /// Analyze code using OpenAI's Codex API
    pub async fn analyze_with_codex(code_snippet: &str, api_key: &str) -> Result<Vec<RiskFinding>> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "Analyze this code for security vulnerabilities, focusing on supply chain, network, context poisoning, and prompt injection risks:\n\n{}",
            code_snippet
        );

        let request_body = serde_json::json!({
            "model": "gpt-4",
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "max_tokens": 1000,
            "temperature": 0.3
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            parse_ai_response(response_text, RiskType::AIBased).await
        } else {
            Ok(vec![])
        }
    }

    /// Analyze code using Google's Gemini API
    pub async fn analyze_with_gemini(code_snippet: &str, api_key: &str) -> Result<Vec<RiskFinding>> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "Identify security vulnerabilities in this code related to supply chain, network, context poisoning, and prompt injection. Respond with structured findings including risk type, severity, description, and evidence:\n\n{}",
            code_snippet
        );

        let request_body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generation_config": {
                "maxOutputTokens": 1000,
                "temperature": 0.3
            }
        });

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent?key={}", api_key);
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            parse_ai_response(response_text, RiskType::AIBased).await
        } else {
            Ok(vec![])
        }
    }

    /// Analyze code using Alibaba Cloud's Qwen API
    pub async fn analyze_with_qwen(code_snippet: &str, api_key: &str, endpoint: &str) -> Result<Vec<RiskFinding>> {
        let client = reqwest::Client::new();
        let prompt = format!(
            "Analyze this code for security vulnerabilities with emphasis on supply chain, network, context poisoning, and prompt injection attacks. Provide structured findings:\n\n{}",
            code_snippet
        );

        let request_body = serde_json::json!({
            "model": "qwen-max",
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "max_tokens": 1000,
            "temperature": 0.3
        });

        let response = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            parse_ai_response(response_text, RiskType::AIBased).await
        } else {
            Ok(vec![])
        }
    }

    /// Generic analysis function that can use any configured AI service
    pub async fn analyze_code_with_ai(
        code_snippet: &str,
        service: &str,
        api_key: &str,
        custom_endpoint: Option<&str>,
    ) -> Result<Vec<RiskFinding>> {
        match service.to_lowercase().as_str() {
            "claude" => Self::analyze_with_claude(code_snippet, api_key).await,
            "codex" | "gpt" => Self::analyze_with_codex(code_snippet, api_key).await,
            "gemini" => Self::analyze_with_gemini(code_snippet, api_key).await,
            "qwen" => {
                if let Some(endpoint) = custom_endpoint {
                    Self::analyze_with_qwen(code_snippet, api_key, endpoint).await
                } else {
                    Err(anyhow::anyhow!("Qwen API requires a custom endpoint"))
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported AI service: {}", service)),
        }
    }
}

/// Helper function to parse AI API responses into RiskFinding structs
async fn parse_ai_response(response: String, default_risk_type: RiskType) -> Result<Vec<RiskFinding>> {
    // Attempt to parse as structured JSON response first
    if let Ok(parsed) = serde_json::from_str::<Vec<HashMap<String, serde_json::Value>>>(&response) {
        let mut findings = Vec::new();
        
        for item in parsed {
            let risk_type = match item.get("risk_type").and_then(|v| v.as_str()) {
                Some("SupplyChain") => RiskType::SupplyChain,
                Some("Network") => RiskType::Network,
                Some("ContextPoisoning") => RiskType::ContextPoisoning,
                Some("PromptInjection") => RiskType::PromptInjection,
                _ => default_risk_type.clone(),
            };
            
            let severity = match item.get("severity").and_then(|v| v.as_str()) {
                Some("High") => Severity::High,
                Some("Medium") => Severity::Medium,
                Some("Low") => Severity::Low,
                _ => Severity::Medium,
            };
            
            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("AI-detected security issue")
                .to_string();
                
            let evidence = item
                .get("evidence")
                .and_then(|v| v.as_str())
                .unwrap_or(&response)
                .to_string();

            findings.push(RiskFinding {
                id: uuid::Uuid::new_v4().to_string(),
                risk_type,
                severity,
                description,
                evidence,
                location: None, // Location is unknown for AI-generated findings
                timestamp: chrono::Utc::now(),
            });
        }
        
        Ok(findings)
    } else {
        // If structured parsing fails, create a generic finding with the response text
        Ok(vec![RiskFinding {
            id: uuid::Uuid::new_v4().to_string(),
            risk_type: default_risk_type,
            severity: Severity::Medium,
            description: "AI analysis completed".to_string(),
            evidence: response,
            location: None, // Location is unknown for AI-generated findings
            timestamp: chrono::Utc::now(),
        }])
    }
}