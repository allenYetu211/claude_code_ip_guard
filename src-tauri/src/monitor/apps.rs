//! 已安装 app 列表（M5 会用到）；M3 留作占位。
//! 通过 NSWorkspace runningApplications 给当前运行 app；
//! 完整的磁盘扫描 (/Applications) 在 M5 实施。

pub use super::nsworkspace::list_running_apps;
