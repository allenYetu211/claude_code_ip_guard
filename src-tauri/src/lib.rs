use std::sync::Arc;
use std::time::Duration;

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub const TRAY_NORMAL_LIGHT: &[u8] = include_bytes!("../icons/tray-normal-light@2x.png");
pub const TRAY_NORMAL_DARK:  &[u8] = include_bytes!("../icons/tray-normal-dark@2x.png");
pub const TRAY_ERROR_LIGHT:  &[u8] = include_bytes!("../icons/tray-error-light@2x.png");
pub const TRAY_ERROR_DARK:   &[u8] = include_bytes!("../icons/tray-error-dark@2x.png");

pub fn tray_icon_bytes(error: bool) -> &'static [u8] {
    // 始终用 dark 变体（白色星）。macOS 在深色菜单栏（绝大多数情况）显示清晰；
    // 即使浅色菜单栏，白色星仍能看到边缘 + 彩色状态点。
    if error { TRAY_ERROR_DARK } else { TRAY_NORMAL_DARK }
}

pub mod ip;
pub mod macos;
pub mod monitor;
pub mod notifier;
pub mod settings;

use crate::notifier::Notifier;
use crate::settings::{Settings, SettingsHandle};

#[tauri::command]
fn ping() -> &'static str {
    "pong"
}

#[tauri::command]
async fn get_ip_report(
    state: tauri::State<'_, AppState>,
    force_refresh: Option<bool>,
) -> Result<ip::IpReport, String> {
    let force = force_refresh.unwrap_or(false);
    if !force {
        if let Some(cached) = state.cache.get_fresh() {
            return Ok(cached);
        }
    }
    let allowed = state.notifier.settings.snapshot().allowed_countries;
    let report = ip::fetch_report(&state.http, &allowed).await;
    state.cache.put(report.clone());
    Ok(report)
}

#[tauri::command]
fn list_running_apps() -> Vec<monitor::InstalledApp> {
    #[cfg(target_os = "macos")]
    {
        monitor::nsworkspace::list_running_apps()
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![]
    }
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Settings {
    state.notifier.settings.snapshot()
}

#[tauri::command]
fn reset_violation_cooldown(
    state: tauri::State<'_, AppState>,
    bundle_id: String,
    country: Option<String>,
) {
    let key = notifier::dedup_key(&bundle_id, &country);
    tracing::info!(?key, "reset_violation_cooldown");
    state.notifier.dedup.clear(&key);
}

#[tauri::command]
fn set_settings(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    next: Settings,
) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    state.cache.set_ttl(Duration::from_secs(next.cache_ttl_secs.max(30)));
    state.notifier.settings.update(next.clone());
    settings::save(&app, &next)?;

    // 同步自启动状态
    let manager = app.autolaunch();
    let currently = manager.is_enabled().unwrap_or(false);
    if next.auto_start && !currently {
        let _ = manager.enable();
    } else if !next.auto_start && currently {
        let _ = manager.disable();
    }
    Ok(())
}

pub struct AppState {
    pub http: reqwest::Client,
    pub cache: Arc<ip::ReportCache>,
    pub notifier: Notifier,
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "打开主界面", true, Some("CmdOrCtrl+O"))?;
    let recheck_item = MenuItem::with_id(app, "recheck", "重新检测 IP", true, Some("CmdOrCtrl+R"))?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出 IP Guard", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(
        app,
        &[&open_item, &recheck_item, &separator1, &separator2, &quit_item],
    )?;

    let icon = Image::from_bytes(tray_icon_bytes(false))
        .unwrap_or_else(|_| app.default_window_icon().cloned().unwrap());

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .icon_as_template(false)
        .tooltip("IP Guard")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "recheck" => {
                let app2 = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app2.state::<AppState>();
                    let allowed = state.notifier.settings.snapshot().allowed_countries;
                    let report = ip::fetch_report(&state.http, &allowed).await;
                    state.cache.put(report.clone());
                    let _ = app2.emit("ip-report-updated", report);
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("ip_guard=info,warn")),
        )
        .try_init();

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("ip-guard/0.1 (macOS)")
        .build()
        .expect("reqwest client");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            ping,
            get_ip_report,
            list_running_apps,
            get_settings,
            set_settings,
            reset_violation_cooldown,
        ])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                // 应用 NSVisualEffectView 毛玻璃到主窗口
                // Popover 材质是 macOS 菜单栏弹窗的标准材质，自动适配浅深色
                if let Some(win) = app.get_webview_window("main") {
                    use window_vibrancy::{
                        apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
                    };
                    let _ = apply_vibrancy(
                        &win,
                        NSVisualEffectMaterial::Popover,
                        Some(NSVisualEffectState::Active),
                        Some(14.0),
                    );

                    // 拦截关闭按钮：隐藏窗口而非退出 app
                    let win_clone = win.clone();
                    win.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            let _ = win_clone.hide();
                        }
                    });
                }
            }

            // 加载或初始化 settings（必须在 store plugin 注册之后）
            let initial = settings::load(app.handle());
            let cache_ttl = Duration::from_secs(initial.cache_ttl_secs.max(30));
            let cache = Arc::new(ip::ReportCache::new(cache_ttl));
            let settings_handle = SettingsHandle::new(initial);
            let notifier = Notifier::new(settings_handle);

            app.manage(AppState { http: http.clone(), cache, notifier });

            build_tray(app.handle())?;

            #[cfg(target_os = "macos")]
            {
                monitor::nsworkspace::start(app.handle().clone());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
