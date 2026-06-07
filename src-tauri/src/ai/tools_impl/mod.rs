/// AI 工具模块
/// 
/// 将各个内置工具拆分到独立文件中，便于维护和扩展。

pub mod open_url;
pub mod web_search;
pub mod summarize_page;
pub mod web_agent;
pub mod run_command;
pub mod dispatcher;

// 重新导出常用类型和函数
pub use dispatcher::execute;
