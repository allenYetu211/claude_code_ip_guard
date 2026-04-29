#[cfg(target_os = "macos")]
pub mod apps;
#[cfg(target_os = "macos")]
pub mod nsworkspace;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationEvent {
    pub bundle_id: Option<String>,
    pub name: Option<String>,
    pub pid: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub bundle_id: String,
    pub name: String,
    pub path: Option<String>,
}

#[cfg(not(target_os = "macos"))]
pub fn start(_app: tauri::AppHandle) {}

#[cfg(not(target_os = "macos"))]
pub fn list_running_apps() -> Vec<InstalledApp> {
    vec![]
}
