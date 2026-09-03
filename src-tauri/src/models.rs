//! Shared data models exchanged between Rust and the frontend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskTier {
    Safe,
    Caution,
    Risky,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: String,
    pub rule_id: String,
    pub category: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub file_count: u64,
    pub last_modified: Option<DateTime<Utc>>,
    pub risk: RiskTier,
    pub deletable: bool,
    pub informational_only: bool,
    pub cleanup_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub current_path: String,
    pub rule_id: String,
    pub items_found: usize,
    pub bytes_found: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanComplete {
    pub item_count: usize,
    pub total_bytes: u64,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub paths: Vec<String>,
    pub permanent: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteItemResult {
    pub path: String,
    pub success: bool,
    pub dry_run: bool,
    pub error: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResult {
    pub results: Vec<DeleteItemResult>,
    pub freed_bytes: u64,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub path: String,
    pub size_bytes: u64,
    pub permanent: bool,
    pub dry_run: bool,
    pub success: bool,
    pub error: Option<String>,
}
