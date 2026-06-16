//! AI 模块 — CoSurf 阅读伴侣和思考搭档核心
//!
//! 从 src-tauri/src/ai/ 迁移，移除所有 Tauri 依赖。
//! 通过 N-API 导出给 Electron 主进程调用。

pub mod provider;
pub mod tools;
pub mod stream;
pub mod skills;
pub mod mcp;
pub mod mcp_manager;
pub mod agent;
pub mod scheduler;      // 智能并行调度器
pub mod context_manager; // 上下文管理器
pub mod checkpoint;     // 检查点管理器

use std::sync::atomic::{AtomicBool, Ordering};

/// 全局取消标志（替代 Tauri AppState.cancel_flag）
pub static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

/// 检查是否被取消
pub fn is_cancelled() -> bool {
    CANCEL_FLAG.load(Ordering::SeqCst)
}

/// 请求取消当前生成
pub fn request_cancel() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

/// 重置取消标志
pub fn reset_cancel() {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
}
