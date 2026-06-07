pub mod agent;
pub mod mcp;
pub mod playwright_client;
pub mod provider;
pub mod sandbox;
pub mod skills;
pub use skills::{Skill, SkillDirInfo};  // 导出 Skill 相关类型
pub mod skills_executors;  // MCP 客户端和命令工具（CLI/Script 已移除）
pub mod stream;
pub mod tools;
pub mod tools_impl;
