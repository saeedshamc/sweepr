//! Safe deletion via trash / Recycle Bin, with dry-run and audit logging.

use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::models::{AuditEntry, DeleteItemResult, DeleteRequest, DeleteResult};
use crate::scanner::assert_not_critical;

pub fn delete_paths(req: DeleteRequest) -> Result<DeleteResult, String> {
    let mut results = Vec::new();
    let mut freed = 0u64;
    let mut failed = 0usize;

    for path_str in &req.paths {
        let path = PathBuf::from(path_str);

        if let Err(e) = assert_not_critical(&path) {
            failed += 1;
            let item = DeleteItemResult {
                path: path_str.clone(),
                success: false,
                dry_run: req.dry_run,
                error: Some(e.clone()),
                size_bytes: 0,
            };
            log_audit(&AuditEntry {
                timestamp: Utc::now(),
                path: path_str.clone(),
                size_bytes: 0,
                permanent: req.permanent,
                dry_run: req.dry_run,
                success: false,
                error: Some(e),
            })?;
            results.push(item);
            continue;
        }

        let size = measure_size_best_effort(&path);

        if req.dry_run {
            freed = freed.saturating_add(size);
            let item = DeleteItemResult {
                path: path_str.clone(),
                success: true,
                dry_run: true,
                error: None,
                size_bytes: size,
            };
            log_audit(&AuditEntry {
                timestamp: Utc::now(),
                path: path_str.clone(),
                size_bytes: size,
                permanent: req.permanent,
                dry_run: true,
                success: true,
                error: None,
            })?;
            results.push(item);
            continue;
        }

        // Permission / existence check — fail gracefully
        if !path.exists() {
            failed += 1;
            let err = "Path does not exist".to_string();
            log_audit(&AuditEntry {
                timestamp: Utc::now(),
                path: path_str.clone(),
                size_bytes: size,
                permanent: req.permanent,
                dry_run: false,
                success: false,
                error: Some(err.clone()),
            })?;
            results.push(DeleteItemResult {
                path: path_str.clone(),
                success: false,
                dry_run: false,
                error: Some(err),
                size_bytes: size,
            });
            continue;
        }

        let delete_result = if req.permanent {
            permanent_delete(&path)
        } else {
            trash::delete(&path).map_err(|e| e.to_string())
        };

        match delete_result {
            Ok(()) => {
                freed = freed.saturating_add(size);
                log_audit(&AuditEntry {
                    timestamp: Utc::now(),
                    path: path_str.clone(),
                    size_bytes: size,
                    permanent: req.permanent,
                    dry_run: false,
                    success: true,
                    error: None,
                })?;
                results.push(DeleteItemResult {
                    path: path_str.clone(),
                    success: true,
                    dry_run: false,
                    error: None,
                    size_bytes: size,
                });
            }
            Err(e) => {
                failed += 1;
                log_audit(&AuditEntry {
                    timestamp: Utc::now(),
                    path: path_str.clone(),
                    size_bytes: size,
                    permanent: req.permanent,
                    dry_run: false,
                    success: false,
                    error: Some(e.clone()),
                })?;
                results.push(DeleteItemResult {
                    path: path_str.clone(),
                    success: false,
                    dry_run: false,
                    error: Some(e),
                    size_bytes: size,
                });
            }
        }
    }

    Ok(DeleteResult {
        results,
        freed_bytes: freed,
        failed,
    })
}

fn permanent_delete(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        fs::remove_file(path).map_err(|e| e.to_string())
    }
}

fn measure_size_best_effort(path: &Path) -> u64 {
    if let Ok(meta) = fs::metadata(path) {
        if meta.is_file() {
            return meta.len();
        }
    }
    let mut total = 0u64;
    for entry in walkdir::WalkDir::new(path).into_iter().flatten() {
        if entry.file_type().is_file() {
            if let Ok(m) = entry.metadata() {
                total = total.saturating_add(m.len());
            }
        }
    }
    total
}

fn audit_log_path() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| "Could not resolve data directory".to_string())?
        .join("sweepr");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("deletion_log.jsonl"))
}

fn log_audit(entry: &AuditEntry) -> Result<(), String> {
    let path = audit_log_path()?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    let line = serde_json::to_string(entry).map_err(|e| e.to_string())?;
    writeln!(file, "{line}").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_audit_log(limit: usize) -> Result<Vec<AuditEntry>, String> {
    let path = audit_log_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut entries: Vec<AuditEntry> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    entries.reverse(); // newest first
    entries.truncate(limit);
    Ok(entries)
}
