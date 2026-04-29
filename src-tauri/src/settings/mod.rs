use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredApp {
    pub bundle_id: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotifyStyle {
    /// 仅原生通知中心
    Toast,
    /// 仅弹出 Dashboard 窗口
    Modal,
    /// 两者都发
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub monitored_apps: Vec<MonitoredApp>,
    pub allowed_countries: Vec<String>,    // ISO alpha-2
    pub cache_ttl_secs: u64,
    pub alert_cooldown_secs: u64,
    pub notify_style: NotifyStyle,
    pub auto_start: bool,
    pub language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            monitored_apps: vec![
                MonitoredApp {
                    bundle_id: "com.anthropic.claudefordesktop".into(),
                    name: "Claude".into(),
                    enabled: true,
                },
                MonitoredApp {
                    bundle_id: "com.openai.chat".into(),
                    name: "ChatGPT".into(),
                    enabled: false,
                },
            ],
            allowed_countries: vec!["US".into(), "JP".into(), "SG".into(), "TW".into()],
            cache_ttl_secs: 300,
            alert_cooldown_secs: 600,
            notify_style: NotifyStyle::Both,
            auto_start: false,
            language: "zh-CN".into(),
        }
    }
}

pub struct SettingsHandle {
    inner: RwLock<Settings>,
}

impl SettingsHandle {
    pub fn new(initial: Settings) -> Arc<Self> {
        Arc::new(Self { inner: RwLock::new(initial) })
    }

    pub fn snapshot(&self) -> Settings {
        self.inner.read().clone()
    }

    pub fn update(&self, s: Settings) {
        *self.inner.write() = s;
    }
}

pub fn load(app: &AppHandle) -> Settings {
    let path = STORE_PATH;
    let store = match app.store(path) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(?e, "failed to open settings store, using defaults");
            return Settings::default();
        }
    };
    match store.get(SETTINGS_KEY) {
        Some(v) => serde_json::from_value(v).unwrap_or_else(|e| {
            tracing::warn!(?e, "settings parse failed, using defaults");
            Settings::default()
        }),
        None => {
            let s = Settings::default();
            store.set(SETTINGS_KEY, serde_json::to_value(&s).unwrap());
            let _ = store.save();
            s
        }
    }
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.set(SETTINGS_KEY, serde_json::to_value(settings).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}
