//! 智能并行调度器（借鉴 Codex）
//!
//! 根据工具类型智能决定并行策略：
//! - Read/Network: 可安全并行
//! - Write/Browser: 需串行执行，避免冲突

use crate::ai::tools::ToolCall;

/// 工具分类
#[derive(Debug, Clone, PartialEq)]
pub enum ToolCategory {
    /// 只读操作（安全并行）
    Read,
    /// 写入操作（需串行，避免文件冲突）
    Write,
    /// 网络请求（可并行）
    Network,
    /// 浏览器操作（需串行，避免标签页冲突）
    Browser,
}

impl ToolCall {
    /// 判断工具类别
    pub fn category(&self) -> ToolCategory {
        match self.name.as_str() {
            // 读取类工具
            "summarize_page" | "translate" => ToolCategory::Read,
            
            // 写入类工具
            "export_markdown" | "run_command" => ToolCategory::Write,
            
            // 网络类工具
            "web_search" => ToolCategory::Network,
            
            // 浏览器类工具
            "open_url" | "web_agent" => ToolCategory::Browser,
            
            // MCP 工具（根据名称判断）
            name if name.starts_with("mcp_") => {
                // MCP 工具通常是网络请求，可并行
                ToolCategory::Network
            }
            
            // Skill 工具（根据名称判断）
            name if name.starts_with("skill_") => {
                // Skill 工具默认视为读取类
                ToolCategory::Read
            }
            
            // 默认视为读取类
            _ => ToolCategory::Read,
        }
    }
}

/// 智能并行调度结果
pub struct ScheduledTools {
    pub read_tools: Vec<ToolCall>,
    pub write_tools: Vec<ToolCall>,
    pub network_tools: Vec<ToolCall>,
    pub browser_tools: Vec<ToolCall>,
}

impl ScheduledTools {
    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.read_tools.is_empty() 
            && self.write_tools.is_empty()
            && self.network_tools.is_empty()
            && self.browser_tools.is_empty()
    }
    
    /// 获取所有工具的总数
    pub fn total_count(&self) -> usize {
        self.read_tools.len() 
            + self.write_tools.len()
            + self.network_tools.len()
            + self.browser_tools.len()
    }
}

/// 智能调度器：将工具调用按类别分组
pub fn schedule_tools(tool_calls: Vec<ToolCall>) -> ScheduledTools {
    let mut scheduled = ScheduledTools {
        read_tools: vec![],
        write_tools: vec![],
        network_tools: vec![],
        browser_tools: vec![],
    };
    
    for tc in tool_calls {
        match tc.category() {
            ToolCategory::Read => scheduled.read_tools.push(tc),
            ToolCategory::Write => scheduled.write_tools.push(tc),
            ToolCategory::Network => scheduled.network_tools.push(tc),
            ToolCategory::Browser => scheduled.browser_tools.push(tc),
        }
    }
    
    tracing::info!(
        "📊 Smart scheduling: read={}, write={}, network={}, browser={}",
        scheduled.read_tools.len(),
        scheduled.write_tools.len(),
        scheduled.network_tools.len(),
        scheduled.browser_tools.len()
    );
    
    scheduled
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_category() {
        let tc = ToolCall {
            id: "1".to_string(),
            name: "summarize_page".to_string(),
            arguments: serde_json::json!({}),
        };
        assert_eq!(tc.category(), ToolCategory::Read);
        
        let tc = ToolCall {
            id: "2".to_string(),
            name: "open_url".to_string(),
            arguments: serde_json::json!({}),
        };
        assert_eq!(tc.category(), ToolCategory::Browser);
        
        let tc = ToolCall {
            id: "3".to_string(),
            name: "mcp_iqs_search".to_string(),
            arguments: serde_json::json!({}),
        };
        assert_eq!(tc.category(), ToolCategory::Network);
    }
    
    #[test]
    fn test_schedule_tools() {
        let tools = vec![
            ToolCall {
                id: "1".to_string(),
                name: "summarize_page".to_string(),
                arguments: serde_json::json!({}),
            },
            ToolCall {
                id: "2".to_string(),
                name: "open_url".to_string(),
                arguments: serde_json::json!({}),
            },
            ToolCall {
                id: "3".to_string(),
                name: "export_markdown".to_string(),
                arguments: serde_json::json!({}),
            },
        ];
        
        let scheduled = schedule_tools(tools);
        assert_eq!(scheduled.read_tools.len(), 1);
        assert_eq!(scheduled.browser_tools.len(), 1);
        assert_eq!(scheduled.write_tools.len(), 1);
        assert_eq!(scheduled.total_count(), 3);
    }
}
