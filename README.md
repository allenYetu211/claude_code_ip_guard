# IP Guard

macOS 菜单栏 IP 守卫。监听指定应用的激活事件，当当前公网 IP 不在你设置的国家白名单内时，弹出原生通知 + Dashboard 提醒。

## 功能

- **多源 IP 检测**：ipwho.is / ipapi.co / ipinfo.io / Cloudflare trace 并发聚合，国家投票合并，5 分钟缓存
- **风险标记**：基于 12 个主流云厂 ASN 静态识别 datacenter，综合 trust score (0-100)
- **服务连通性**：HEAD 探测 claude.ai / anthropic.com，显示状态码 + 延迟
- **应用激活监听**：原生 NSWorkspace observer，bundleId 精准匹配
- **国家白名单**：ISO alpha-2，多个允许地区
- **提醒方式**：原生通知 / Dashboard 弹窗 / 两者都发
- **去抖**：同一 (app, 国家) 组合 10 分钟内只提醒一次；点击关闭可立即重置
- **菜单栏常驻**：无 Dock 图标，托盘 tooltip + title 实时反映状态（✓ 正常 / ⚠ 注意 / ⨯ 违规 / ○ 离线）
- **开机自启动**：可在设置中开关

## 开发

```bash
pnpm install
pnpm tauri dev
```

启动后菜单栏出现图标，主窗口默认隐藏，从托盘菜单 → Open Dashboard 打开。

## 打包

```bash
pnpm tauri build
```

产物：`src-tauri/target/release/bundle/dmg/IP Guard_<version>_aarch64.dmg`

未做 Apple notarization；首次运行需在 系统设置 → 隐私与安全性 中点 "仍要打开"。

## 项目结构

```
src-tauri/src/
├── lib.rs                    # 入口、命令注册、托盘
├── ip/
│   ├── mod.rs                # IpReport 数据结构 + trust score
│   ├── aggregator.rs         # 多源并发 + 国家投票合并 + datacenter ASN 命中
│   ├── cache.rs              # TTL 缓存
│   ├── connectivity.rs       # claude.ai / anthropic.com HEAD 探测
│   └── providers/            # ipwho / ipapi.co / ipinfo / cloudflare
├── monitor/
│   ├── mod.rs                # ActivationEvent / InstalledApp 类型
│   └── nsworkspace.rs        # NSWorkspaceDidActivateApplicationNotification observer
├── notifier.rs               # 违规判定 + 通知 + 弹窗 + 去抖 + 托盘状态
└── settings/
    └── mod.rs                # Settings 结构 + 持久化（tauri-plugin-store）

src/
├── App.tsx
├── components/{IpReportCard, SettingsPanel, ViolationBanner}.tsx
├── hooks/{useIpReport, useSettings, useTauriEvent}.ts
└── types.ts                  # 前后端共享类型
```

## 后端测试

```bash
cd src-tauri && cargo test
```

包含 merge / trust score 等单元测试。

## 实现要点

### NSWorkspace observer

`Retained<ProtocolObject<dyn NSObjectProtocol>>` 不是 Send/Sync，无法放进 `OnceLock`。
解决：故意 `mem::forget` 让 observer 随进程存活，配合 `AtomicBool` 守护重复安装。

### 多源国家投票

```text
≥2 个 provider 国家一致 → 取该国家
否则 → 第一个非空
```

### 去抖

`HashMap<(bundleId, country), Instant>`，cooldown 默认 600s。
前端点击违规条会调用 `reset_violation_cooldown` 立即清除该 key。

## 许可

MIT
# claude_code_ip_guard
