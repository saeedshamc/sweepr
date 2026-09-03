//! Filesystem scanner — walks rule targets and streams results to the UI.

use chrono::{DateTime, Duration, Utc};
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::models::{RiskTier, ScanComplete, ScanItem, ScanProgress};
use crate::rules::{expand_paths, is_protected, load_rules, PlatformRules, ScanRule};

pub fn run_scan(app: AppHandle) -> Result<ScanComplete, String> {
    let started = Instant::now();
    let platform = load_rules()?;
    let mut items: Vec<ScanItem> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut bytes_found: u64 = 0;

    for rule in &platform.rules {
        let _ = app.emit(
            "scan-progress",
            ScanProgress {
                current_path: rule.label.clone(),
                rule_id: rule.id.clone(),
                items_found: items.len(),
                bytes_found,
            },
        );

        match scan_rule(&app, &platform, rule, &mut errors) {
            Ok(found) => {
                for item in found {
                    bytes_found = bytes_found.saturating_add(item.size_bytes);
                    let _ = app.emit("scan-item", &item);
                    items.push(item);
                }
            }
            Err(e) => errors.push(format!("{}: {e}", rule.id)),
        }
    }

    let complete = ScanComplete {
        item_count: items.len(),
        total_bytes: bytes_found,
        duration_ms: started.elapsed().as_millis() as u64,
        errors,
    };
    let _ = app.emit("scan-complete", &complete);
    Ok(complete)
}

fn scan_rule(
    app: &AppHandle,
    platform: &PlatformRules,
    rule: &ScanRule,
    errors: &mut Vec<String>,
) -> Result<Vec<ScanItem>, String> {
    if let Some(tool) = &rule.tool {
        return Ok(scan_tool(rule, tool));
    }

    if !rule.match_dir_names.is_empty() {
        return Ok(discover_named_dirs(app, platform, rule, errors));
    }

    let paths = expand_paths(&rule.paths);
    let mut out = Vec::new();

    for path in paths {
        if is_protected(&path, &platform.protected_paths) {
            errors.push(format!(
                "Skipped protected path: {}",
                path.display()
            ));
            continue;
        }

        if !path.exists() {
            continue;
        }

        if rule.list_children {
            out.extend(list_aged_children(app, platform, rule, &path, errors));
        } else {
            match measure_path(&path, rule.recursive) {
                Ok((size, count, mtime)) => {
                    if !passes_age(mtime, rule.min_age_days) && rule.min_age_days > 0 {
                        // Still show the location if it has size; age filter is soft for aggregates
                        if size == 0 {
                            continue;
                        }
                    }
                    out.push(make_item(rule, &rule.label, &path, size, count, mtime));
                }
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }
        }

        let _ = app.emit(
            "scan-progress",
            ScanProgress {
                current_path: path.display().to_string(),
                rule_id: rule.id.clone(),
                items_found: out.len(),
                bytes_found: out.iter().map(|i| i.size_bytes).sum(),
            },
        );
    }

    Ok(out)
}

fn list_aged_children(
    app: &AppHandle,
    platform: &PlatformRules,
    rule: &ScanRule,
    parent: &Path,
    errors: &mut Vec<String>,
) -> Vec<ScanItem> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(e) => {
            errors.push(format!("{}: {e}", parent.display()));
            return out;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_protected(&path, &platform.protected_paths) {
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let mtime = meta_mtime(&meta);
        if !passes_age(mtime, rule.min_age_days) {
            continue;
        }

        match measure_path(&path, true) {
            Ok((size, count, mt)) => {
                if size == 0 {
                    continue;
                }
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                out.push(make_item(rule, &name, &path, size, count, mt));
            }
            Err(e) => errors.push(format!("{}: {e}", path.display())),
        }

        let _ = app.emit(
            "scan-progress",
            ScanProgress {
                current_path: path.display().to_string(),
                rule_id: rule.id.clone(),
                items_found: out.len(),
                bytes_found: out.iter().map(|i| i.size_bytes).sum(),
            },
        );
    }
    out
}

fn discover_named_dirs(
    app: &AppHandle,
    platform: &PlatformRules,
    rule: &ScanRule,
    errors: &mut Vec<String>,
) -> Vec<ScanItem> {
    let roots = expand_paths(&rule.paths);
    let names: Vec<String> = rule.match_dir_names.clone();
    let skip: Vec<String> = rule.skip_dir_names.clone();
    let mut out = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }
        if is_protected(&root, &platform.protected_paths) {
            continue;
        }

        let names_filter = names.clone();
        let skip_filter = skip.clone();
        let walker = WalkDir::new(&root)
            .max_depth(rule.max_depth)
            .skip_hidden(false)
            .process_read_dir(move |_depth, _path, _state, children| {
                children.retain(|entry| {
                    if let Ok(e) = entry {
                        let name = e.file_name.to_string_lossy();
                        // Don't descend into skip dirs (unless they are themselves a match target).
                        if skip_filter.iter().any(|s| s == &name)
                            && !names_filter.iter().any(|n| n == &name)
                        {
                            return false;
                        }
                    }
                    true
                });
            });

        for entry in walker.into_iter().flatten() {
            if !entry.file_type().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if !names.iter().any(|n| n == &name) {
                continue;
            }

            let path = entry.path();
            if is_protected(&path, &platform.protected_paths) {
                continue;
            }

            // Avoid measuring nested node_modules inside another node_modules
            if name == "node_modules" {
                if let Some(parent) = path.parent() {
                    if parent.components().any(|c| c.as_os_str() == "node_modules") {
                        continue;
                    }
                }
            }

            let meta = match std::fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(format!("{}: {e}", path.display()));
                    continue;
                }
            };
            let mtime = meta_mtime(&meta);
            if !passes_age(mtime, rule.min_age_days) {
                continue;
            }

            match measure_path(&path, true) {
                Ok((size, count, mt)) => {
                    if size == 0 {
                        continue;
                    }
                    let label = format!("{} ({})", name, path.display());
                    out.push(make_item(rule, &label, &path, size, count, mt));
                }
                Err(e) => errors.push(format!("{}: {e}", path.display())),
            }

            let _ = app.emit(
                "scan-progress",
                ScanProgress {
                    current_path: path.display().to_string(),
                    rule_id: rule.id.clone(),
                    items_found: out.len(),
                    bytes_found: out.iter().map(|i| i.size_bytes).sum(),
                },
            );
        }
    }

    out
}

fn scan_tool(rule: &ScanRule, tool: &str) -> Vec<ScanItem> {
    match tool {
        "docker" => scan_docker(rule),
        _ => Vec::new(),
    }
}

fn scan_docker(rule: &ScanRule) -> Vec<ScanItem> {
    let output = Command::new("docker").args(["system", "df"]).output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let size = parse_docker_df_total(&stdout);
            vec![ScanItem {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                category: rule.category.clone(),
                name: rule.label.clone(),
                path: "(docker CLI — not a filesystem path)".into(),
                size_bytes: size,
                file_count: 0,
                last_modified: None,
                risk: rule.risk.clone(),
                deletable: false,
                informational_only: true,
                cleanup_hint: rule.cleanup_hint.clone().or_else(|| {
                    Some(
                        "Run: docker system prune -a  (or image/builder prune). Never delete Docker storage files manually."
                            .into(),
                    )
                }),
            }]
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            vec![info_item(
                rule,
                "Docker not available or daemon not running",
                &format!("docker system df failed: {err}"),
            )]
        }
        Err(_) => vec![info_item(
            rule,
            "Docker CLI not found",
            "Install Docker and ensure `docker` is on PATH. Cleanup must go through Docker CLI.",
        )],
    }
}

fn info_item(rule: &ScanRule, name: &str, hint: &str) -> ScanItem {
    ScanItem {
        id: Uuid::new_v4().to_string(),
        rule_id: rule.id.clone(),
        category: rule.category.clone(),
        name: name.into(),
        path: "(informational)".into(),
        size_bytes: 0,
        file_count: 0,
        last_modified: None,
        risk: RiskTier::Risky,
        deletable: false,
        informational_only: true,
        cleanup_hint: Some(hint.into()),
    }
}

/// Rough parse of `docker system df` reclaimable / total size lines.
fn parse_docker_df_total(stdout: &str) -> u64 {
    let mut total = 0u64;
    for line in stdout.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // TYPE TOTAL ACTIVE SIZE RECLAIMABLE
        if cols.len() >= 4 {
            total = total.saturating_add(parse_size_string(cols[3]));
        }
    }
    total
}

fn parse_size_string(s: &str) -> u64 {
    let s = s.trim();
    let (num, mult) = if let Some(v) = s.strip_suffix("GB") {
        (v, 1_000_000_000f64)
    } else if let Some(v) = s.strip_suffix("MB") {
        (v, 1_000_000f64)
    } else if let Some(v) = s.strip_suffix("KB") {
        (v, 1_000f64)
    } else if let Some(v) = s.strip_suffix('B') {
        (v, 1f64)
    } else {
        (s, 1f64)
    };
    num.parse::<f64>()
        .map(|n| (n * mult) as u64)
        .unwrap_or(0)
}

fn measure_path(path: &Path, recursive: bool) -> Result<(u64, u64, Option<DateTime<Utc>>), String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let mtime = meta_mtime(&meta);

    if meta.is_file() {
        return Ok((meta.len(), 1, mtime));
    }

    if !recursive {
        // Sum immediate children only
        let mut size = 0u64;
        let mut count = 0u64;
        if let Ok(rd) = std::fs::read_dir(path) {
            for e in rd.flatten() {
                if let Ok(m) = e.metadata() {
                    if m.is_file() {
                        size += m.len();
                        count += 1;
                    } else if m.is_dir() {
                        count += 1;
                    }
                }
            }
        }
        return Ok((size, count, mtime));
    }

    let mut size = 0u64;
    let mut count = 0u64;
    for entry in WalkDir::new(path).skip_hidden(false).into_iter().flatten() {
        if entry.file_type().is_file() {
            if let Ok(m) = entry.metadata() {
                size = size.saturating_add(m.len());
                count += 1;
            }
        }
    }
    Ok((size, count, mtime))
}

fn meta_mtime(meta: &std::fs::Metadata) -> Option<DateTime<Utc>> {
    meta.modified().ok().map(|t| DateTime::<Utc>::from(t))
}

fn passes_age(mtime: Option<DateTime<Utc>>, min_age_days: u64) -> bool {
    if min_age_days == 0 {
        return true;
    }
    match mtime {
        Some(mt) => Utc::now() - mt >= Duration::days(min_age_days as i64),
        None => true,
    }
}

fn make_item(
    rule: &ScanRule,
    name: &str,
    path: &Path,
    size: u64,
    count: u64,
    mtime: Option<DateTime<Utc>>,
) -> ScanItem {
    let informational = rule.informational_only || rule.risk == RiskTier::Risky;
    ScanItem {
        id: Uuid::new_v4().to_string(),
        rule_id: rule.id.clone(),
        category: rule.category.clone(),
        name: name.to_string(),
        path: path.display().to_string(),
        size_bytes: size,
        file_count: count,
        last_modified: mtime,
        risk: rule.risk.clone(),
        deletable: rule.deletable && !informational,
        informational_only: informational,
        cleanup_hint: rule.cleanup_hint.clone(),
    }
}

/// Ensure critical system paths never become delete candidates.
pub fn assert_not_critical(path: &Path) -> Result<(), String> {
    let critical: &[&str] = if cfg!(target_os = "windows") {
        &[
            r"C:\Windows\System32",
            r"C:\Windows\SysWOW64",
            r"C:\Windows\WinSxS",
            r"C:\Windows\Boot",
        ]
    } else {
        &["/", "/bin", "/sbin", "/usr", "/lib", "/boot", "/etc"]
    };

    let s = path.to_string_lossy();
    for c in critical {
        let cpath = PathBuf::from(c);
        if path == cpath || (c != &"/" && s.starts_with(&format!("{}{}", c, std::path::MAIN_SEPARATOR))) {
            if *c == "/" && path.parent().is_some() {
                continue;
            }
            return Err(format!("Refusing to delete critical path: {s}"));
        }
    }

    // Never delete drive roots
    if path.parent().is_none() {
        return Err(format!("Refusing to delete filesystem root: {s}"));
    }

    Ok(())
}
