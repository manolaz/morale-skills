use crate::database::connection::Database;
use crate::models::{AISkill, RiskFinding, RiskType, Severity, AuditLog};
use anyhow::Result;
use std::collections::HashMap;

/// Utility functions for database operations
/// These functions have been moved to queries.rs to fix trait bound issues with SurrealDB
/// This file exists to maintain the module structure and can be used for future utility functions
impl Database {

}
