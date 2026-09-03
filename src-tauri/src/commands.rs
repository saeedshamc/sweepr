//! Tauri command handlers bridging the React UI and Rust engine.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::deleter::{delete_paths, read_audit_log};
use crate::models::{AuditEntry, DeleteRequest, DeleteResult, ScanComplete};
use crate::rules::{current_platform_key, load_rules, ScanRule};
use crate::scanner;

pub struct ScanState {
    pub running: AtomicBool,
}

impl Default for ScanState {
    fn default() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }
}

#[tauri::command]
pub fn get_platform() -> String {
    current_platform_key().to_string()
}

#[tauri::command]
pub fn get_scan_rules() -> Result<Vec<ScanRule>, String> {
    Ok(load_rules()?.rules)
}

#[tauri::command]
pub fn start_scan(app: AppHandle, state: State<'_, Arc<ScanState>>) -> Result<(), String> {
    if state
        .running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("A scan is already running".into());
    }

    let flag = Arc::clone(&state);
    std::thread::spawn(move || {
        let _ = scanner::run_scan(app);
        flag.running.store(false, Ordering::SeqCst);
    });

    Ok(())
}

#[tauri::command]
pub fn get_last_scan_summary() -> Result<Option<ScanComplete>, String> {
    // Summary is streamed via events; retained for API symmetry.
    Ok(None)
}

#[tauri::command]
pub fn delete_selected(req: DeleteRequest) -> Result<DeleteResult, String> {
    if req.paths.is_empty() {
        return Err("No paths provided".into());
    }
    // Run on a blocking pool mindset — command itself is sync; UI awaits it.
    delete_paths(req)
}

#[tauri::command]
pub fn get_audit_log(limit: Option<usize>) -> Result<Vec<AuditEntry>, String> {
    read_audit_log(limit.unwrap_or(100))
}

#[tauri::command]
pub fn is_scan_running(state: State<'_, Arc<ScanState>>) -> bool {
    state.running.load(Ordering::SeqCst)
}
