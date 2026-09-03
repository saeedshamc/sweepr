//! Scan rules loader — schema lives in JSON so new targets need no scanner rebuild.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::RiskTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesFile {
    pub version: u32,
    pub platforms: HashMap<String, PlatformRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformRules {
    pub protected_paths: Vec<String>,
    pub rules: Vec<ScanRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRule {
    pub id: String,
    pub category: String,
    pub label: String,
    #[serde(default)]
    pub paths: Vec<String>,
    pub risk: RiskTier,
    #[serde(default)]
    pub min_age_days: u64,
    #[serde(default = "default_true")]
    pub deletable: bool,
    #[serde(default)]
    pub informational_only: bool,
    #[serde(default = "default_true")]
    pub recursive: bool,
    /// When true, emit one item per direct child instead of the folder itself.
    #[serde(default)]
    pub list_children: bool,
    /// Discover directories with these names under `paths` roots.
    #[serde(default)]
    pub match_dir_names: Vec<String>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default)]
    pub skip_dir_names: Vec<String>,
    #[serde(default)]
    pub cleanup_hint: Option<String>,
    /// Optional external tool probe (e..g. `"docker"`).
    #[serde(default)]
    pub tool: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_max_depth() -> usize {
    4
}

/// Embedded fallback so the app works even if the resource file is missing.
const EMBEDDED_RULES: &str = include_str!("../resources/scan_rules.json");

pub fn current_platform_key() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        // macOS can reuse Linux-style home caches for now
        "linux"
    } else {
        "linux"
    }
}

pub fn load_rules() -> Result<PlatformRules, String> {
    let key = current_platform_key();
    let file = load_rules_file()?;
    file.platforms
        .get(key)
        .cloned()
        .ok_or_else(|| format!("No scan rules defined for platform '{key}'"))
}

fn load_rules_file() -> Result<RulesFile, String> {
    // Prefer a user-editable copy next to the executable / in app config.
    if let Some(path) = user_rules_path() {
        if path.exists() {
            let text = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
            return serde_json::from_str(&text)
                .map_err(|e| format!("Invalid rules JSON in {}: {e}", path.display()));
        }
    }

    // Bundled resource (release installs)
    if let Ok(resource) = resource_rules_path() {
        if resource.exists() {
            let text = fs::read_to_string(&resource)
                .map_err(|e| format!("Failed to read {}: {e}", resource.display()))?;
            return serde_json::from_str(&text)
                .map_err(|e| format!("Invalid bundled rules JSON: {e}"));
        }
    }

    // Dev / fallback
    serde_json::from_str(EMBEDDED_RULES).map_err(|e| format!("Invalid embedded rules: {e}"))
}

fn user_rules_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sweepr").join("scan_rules.json"))
}

fn resource_rules_path() -> Result<PathBuf, String> {
    // During `tauri dev`, resources live under src-tauri/resources
    let candidates = [
        PathBuf::from("resources/scan_rules.json"),
        PathBuf::from("src-tauri/resources/scan_rules.json"),
    ];
    for c in candidates {
        if c.exists() {
            return Ok(c);
        }
    }
    Err("bundled rules not found".into())
}

/// Expand `~`, `%VAR%`, and `$VAR` style placeholders.
pub fn expand_path(raw: &str) -> PathBuf {
    let mut s = raw.to_string();

    if s.starts_with("~/") || s == "~" {
        if let Some(home) = dirs::home_dir() {
            s = if s == "~" {
                home.to_string_lossy().into_owned()
            } else {
                home.join(&s[2..]).to_string_lossy().into_owned()
            };
        }
    }

    // Windows %ENV% expansion
    while let Some(start) = s.find('%') {
        let rest = &s[start + 1..];
        if let Some(end_rel) = rest.find('%') {
            let var = &rest[..end_rel];
            if !var.is_empty() {
                if let Ok(val) = env::var(var) {
                    s = format!("{}{}{}", &s[..start], val, &rest[end_rel + 1..]);
                    continue;
                }
            }
            break;
        } else {
            break;
        }
    }

    PathBuf::from(s)
}

pub fn expand_paths(paths: &[String]) -> Vec<PathBuf> {
    paths.iter().map(|p| expand_path(p)).collect()
}

pub fn is_protected(path: &Path, protected: &[String]) -> bool {
    let expanded: Vec<PathBuf> = protected.iter().map(|p| expand_path(p)).collect();
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    for p in &expanded {
        let p_canon = p.canonicalize().unwrap_or_else(|_| p.clone());
        if canon == p_canon || canon.starts_with(&p_canon) {
            // Exact protected root (e.g. "/") would block everything — only exact match for roots
            if is_filesystem_root(p) {
                if canon == p_canon {
                    return true;
                }
                continue;
            }
            return true;
        }
    }
    false
}

fn is_filesystem_root(path: &Path) -> bool {
    path.parent().is_none()
        || path
            .to_string_lossy()
            .trim_end_matches(['/', '\\'])
            .chars()
            .count()
            <= 3
            && path.to_string_lossy().contains(':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_rules_parse() {
        let file: RulesFile = serde_json::from_str(EMBEDDED_RULES).unwrap();
        assert!(file.platforms.contains_key("windows"));
        assert!(file.platforms.contains_key("linux"));
    }
}
