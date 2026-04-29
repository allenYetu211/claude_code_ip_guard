//! macOS NSWorkspace 应用激活观察者。
//!
//! 关键点：
//! 1. observer token 必须长期持有，否则 block 释放后回调丢失。
//!    `Retained<ProtocolObject<dyn NSObjectProtocol>>` 不是 Send/Sync，
//!    无法放进 `OnceLock`，所以故意 forget 让 retain count +1 随进程存活，
//!    并用 AtomicBool 守护重复安装。
//! 2. block 在 main queue 上跑，可安全访问 AppKit；用 `frontmostApplication()`
//!    比解析 userInfo 字典更稳。
//! 3. 通过 `tauri::Emitter::emit` 把激活事件发到前端，绕过 Send/Sync 问题。

use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceDidActivateApplicationNotification,
};
use objc2_foundation::{NSNotification, NSOperationQueue};
use tauri::{AppHandle, Emitter};
use tracing::{debug, info};

use super::{ActivationEvent, InstalledApp};

static INSTALLED: AtomicBool = AtomicBool::new(false);

pub fn start(app: AppHandle) {
    if INSTALLED.swap(true, Ordering::SeqCst) {
        debug!("nsworkspace observer already installed");
        return;
    }

    let workspace = NSWorkspace::sharedWorkspace();
    let center = workspace.notificationCenter();
    let main_queue = NSOperationQueue::mainQueue();

    let app_handle = app.clone();
    let block = RcBlock::new(move |_notif: NonNull<NSNotification>| {
        let workspace = NSWorkspace::sharedWorkspace();
        let event = read_frontmost(&workspace);
        debug!(?event, "app activated");
        let _ = app_handle.emit("app-activated", event.clone());

        // 转入 tokio 异步运行时跑 notifier 逻辑（IP 检测可能阻塞）
        let app_for_notifier = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            crate::notifier::handle_activation(app_for_notifier, event).await;
        });
    });

    let token = unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceDidActivateApplicationNotification),
            None,
            Some(&main_queue),
            &block,
        )
    };
    // 故意泄漏：观察者随进程存活，无需移除
    std::mem::forget(token);
    info!("nsworkspace observer installed");
}

fn read_frontmost(workspace: &NSWorkspace) -> ActivationEvent {
    let mut event = ActivationEvent {
        bundle_id: None,
        name: None,
        pid: 0,
    };
    if let Some(app) = workspace.frontmostApplication() {
        event = running_to_event(&app);
    }
    event
}

fn running_to_event(app: &NSRunningApplication) -> ActivationEvent {
    let bundle_id = app.bundleIdentifier().map(|s| s.to_string());
    let name = app.localizedName().map(|s| s.to_string());
    let pid = app.processIdentifier();
    ActivationEvent { bundle_id, name, pid }
}

pub fn list_running_apps() -> Vec<InstalledApp> {
    let workspace = NSWorkspace::sharedWorkspace();
    let arr = workspace.runningApplications();
    let mut out = Vec::new();
    for i in 0..arr.len() {
        let app = arr.objectAtIndex(i);
        // 过滤后台/无 GUI 进程：activationPolicy != regular(0) 跳过
        if app.activationPolicy().0 != 0 {
            continue;
        }
        let bundle_id = app.bundleIdentifier().map(|s| s.to_string());
        let name = app.localizedName().map(|s| s.to_string());
        let url = app.bundleURL();
        let path = url.and_then(|u| u.path()).map(|s| s.to_string());
        if let (Some(bid), Some(n)) = (bundle_id, name) {
            out.push(InstalledApp { bundle_id: bid, name: n, path });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out.dedup_by(|a, b| a.bundle_id == b.bundle_id);
    out
}
