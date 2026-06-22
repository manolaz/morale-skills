pub mod supply_chain;
pub mod network;
pub mod context_poisoning;
pub mod prompt_injection;
pub mod ai_integration;
pub mod tools_permissions;  // New module for AI agent skills tools permissions
pub mod utils;
pub mod base;  // Base trait for risk checkers

pub use supply_chain::SupplyChainChecker;
pub use network::NetworkChecker;
pub use context_poisoning::ContextPoisoningChecker;
pub use prompt_injection::PromptInjectionChecker;
pub use ai_integration::AIAnalyzer;
pub use tools_permissions::ToolsPermissionsChecker;  // New checker
pub use utils::is_text_file;
pub use base::RiskChecker;  // Export the base trait