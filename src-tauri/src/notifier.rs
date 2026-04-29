use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tracing::{debug, info, warn};

use crate::ip;
use crate::monitor::ActivationEvent;
use crate::settings::{NotifyStyle, Settings, SettingsHandle};
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ViolationPayload {
    pub bundle_id: String,
    pub app_name: String,
    pub country: Option<String>,
    pub country_name: Option<String>,
    pub ip: Option<String>,
    pub trust_score: u8,
    pub allowed_countries: Vec<String>,
}

#[derive(Default)]
pub struct DedupCache {
    inner: Mutex<HashMap<String, Instant>>,
}

impl DedupCache {
    pub fn should_alert(&self, key: &str, cooldown: Duration) -> bool {
        let mut g = self.inner.lock();
        let now = Instant::now();
        if let Some(last) = g.get(key) {
            if now.duration_since(*last) < cooldown {
                return false;
            }
        }
        g.insert(key.to_string(), now);
        true
    }

    pub fn clear(&self, key: &str) {
        self.inner.lock().remove(key);
    }

    pub fn clear_all(&self) {
        self.inner.lock().clear();
    }
}

pub fn dedup_key(bundle_id: &str, country: &Option<String>) -> String {
    format!("{}|{}", bundle_id, country.as_deref().unwrap_or("unknown"))
}

pub struct Notifier {
    pub settings: Arc<SettingsHandle>,
    pub dedup: Arc<DedupCache>,
}

impl Notifier {
    pub fn new(settings: Arc<SettingsHandle>) -> Self {
        Self { settings, dedup: Arc::new(DedupCache::default()) }
    }
}

/// 处理一次应用激活事件：查白名单、按需触发 IP 检测、判定违规并提醒。
pub async fn handle_activation(app: AppHandle, event: ActivationEvent) {
    let bundle_id = match event.bundle_id.clone() {
        Some(b) => b,
        None => return,
    };
    // 跳过自身，避免每次切回 Dashboard 都触发
    if bundle_id == "ip-guard" || bundle_id.contains("com.utauu.ipguard") {
        return;
    }

    let state = app.state::<AppState>();
    let settings = state.notifier.settings.snapshot();

    let monitored = settings
        .monitored_apps
        .iter()
        .find(|m| m.enabled && m.bundle_id == bundle_id);
    let monitored = match monitored {
        Some(m) => m.clone(),
        None => return,
    };

    debug!(?bundle_id, "monitored app activated");

    // 监控应用激活时强制重新检测（不走缓存），并把结果写回缓存供 Dashboard 读取
    let report = ip::fetch_report(&state.http, &settings.allowed_countries).await;
    state.cache.put(report.clone());
    let _ = app.emit("ip-report-updated", report.clone());

    // 更新托盘状态
    update_tray_state(&app, &report, &settings.allowed_countries);

    // 判定是否合规
    let country = report.country.clone();
    let in_allowlist = match &country {
        Some(c) => settings
            .allowed_countries
            .iter()
            .any(|a| a.eq_ignore_ascii_case(c)),
        None => false,
    };

    if in_allowlist {
        debug!(?country, "allowed");
        return;
    }

    // 去抖：同 (app, country) 在 cooldown 内只提醒一次（前端 dismiss 时会清除）
    let key = dedup_key(&bundle_id, &country);
    let cooldown = Duration::from_secs(settings.alert_cooldown_secs);
    if !state.notifier.dedup.should_alert(&key, cooldown) {
        info!(?key, "alert suppressed by cooldown");
        return;
    }

    info!(?bundle_id, ?country, "VIOLATION: alerting");

    let payload = ViolationPayload {
        bundle_id: bundle_id.clone(),
        app_name: monitored.name.clone(),
        country: country.clone(),
        country_name: report.country_name.clone(),
        ip: report.ip.clone(),
        trust_score: report.trust_score,
        allowed_countries: settings.allowed_countries.clone(),
    };

    dispatch_alert(&app, &settings, &payload);
    let _ = app.emit("ip-violation", payload);
}

fn dispatch_alert(app: &AppHandle, settings: &Settings, p: &ViolationPayload) {
    if matches!(settings.notify_style, NotifyStyle::Toast | NotifyStyle::Both) {
        let title = format!("⚠️ {} 区域异常", p.app_name);
        let body = format!(
            "当前 IP {} 位于 {}，不在允许地区 ({})",
            p.ip.as_deref().unwrap_or("未知"),
            p.country_name.as_deref().or(p.country.as_deref()).unwrap_or("未知"),
            p.allowed_countries.join("/"),
        );
        if let Err(e) = app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            warn!(?e, "send notification failed");
        }
    }
    if matches!(settings.notify_style, NotifyStyle::Modal | NotifyStyle::Both) {
        if let Some(win) = app.get_webview_window("main") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

/// 根据当前报告 + 白名单更新托盘图标 + tooltip
pub fn update_tray_state(app: &AppHandle, report: &ip::IpReport, allowed: &[String]) {
    let in_allow = match &report.country {
        Some(c) => allowed.iter().any(|a| a.eq_ignore_ascii_case(c)) || allowed.is_empty(),
        None => true, // 没拿到国家时不强制告警
    };

    let icon_bytes = crate::tray_icon_bytes(!in_allow);
    let tip = if in_allow { "正常" } else { "IP 不在允许地区" };

    if let Some(tray) = app.tray_by_id("main-tray") {
        let country = report.country.clone().unwrap_or_else(|| "??".into());
        let tooltip = format!("IP Guard · {} · {}", country, tip);
        let _ = tray.set_tooltip(Some(&tooltip));
        if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
            let _ = tray.set_icon(Some(img));
        }
        // 不再用 title 文本，图标已传达状态
        let _ = tray.set_title(None::<&str>);
    }
}
