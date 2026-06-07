/// web_search 工具实现
/// 
/// 使用阿里云 IQS (Intelligent Query Service) API 进行网络搜索

use tauri::{AppHandle, Manager};
use reqwest::Client;
use std::time::Duration;
use tracing::{info, error};

use crate::error::{AppError, AppResult};
use crate::ai::tools::{ToolCall, ToolResult};
use crate::state::AppState;

/// 执行 web_search 工具
pub async fn execute(app: &AppHandle, tool_call: &ToolCall) -> AppResult<ToolResult> {
    info!("🔍 Executing web_search tool (IQS)");
    
    let args = &tool_call.arguments;
    
    // 提取参数
    let query = args.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("Missing query parameter".into()))?;
    
    let engine_type = args.get("engine_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Generic");
    
    let time_range = args.get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("OneWeek");
    
    let max_results = args.get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    
    info!("  query: {}", query);
    info!("  engine_type: {}", engine_type);
    info!("  time_range: {}", time_range);
    info!("  max_results: {}", max_results);
    
    // 获取 IQS API Key
    let api_key = if let Some(state) = app.try_state::<AppState>() {
        if let Ok(db) = state.db.lock() {
            db.get_setting("iqs.api_key")
                .ok()
                .flatten()
                .filter(|k| !k.is_empty())
        } else {
            None
        }
    } else {
        None
    };
    
    if api_key.is_none() {
        return Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: "错误：未配置阿里云 IQS API Key。请在设置 → 工具中配置 ALIYUN_IQS_API_KEY。\n\n获取方式：访问 https://help.aliyun.com/zh/document_detail/3025781.html".to_string(),
            success: false,
        });
    }
    
    let api_key = api_key.unwrap();
    
    // 调用 IQS API
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;
    
    // 正确的 IQS API 端点
    let iqs_url = "https://cloud-iqs.aliyuncs.com/search/unified";
    
    // 构建请求体（根据阿里云 IQS API 文档）
    let request_body = serde_json::json!({
        "query": query,
        "engineType": engine_type,
        "timeRange": time_range,
        "advancedParams": {
            "numResults": max_results
        }
    });
    
    info!("📤 Sending request to IQS API...");
    
    let response = client.post(iqs_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("IQS API request failed: {}", e)))?;
    
    let status = response.status();
    info!("📥 IQS API response status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("❌ IQS API error: {} - {}", status, error_text);
        return Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output: format!("IQS API 请求失败 ({}): {}", status, error_text),
            success: false,
        });
    }
    
    let result: serde_json::Value = response.json().await
        .map_err(|e| AppError::Internal(format!("Failed to parse IQS response: {}", e)))?;
    
    info!("📥 IQS API response: {:?}", result);
    
    // 解析结果（根据阿里云 IQS API 文档）
    // 返回格式：{ "items": [{ "title": "...", "link": "...", "summary": "..." }] }
    if let Some(items) = result.get("items").and_then(|r| r.as_array()) {
        if items.is_empty() {
            return Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: "未找到相关搜索结果。".to_string(),
                success: true,
            });
        }
        
        // 格式化输出
        let formatted_results: Vec<String> = items.iter().enumerate().map(|(i, item)| {
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("N/A");
            let url = item.get("link").and_then(|u| u.as_str()).unwrap_or("N/A");
            let summary = item.get("summary").and_then(|s| s.as_str()).unwrap_or("N/A");
            
            format!("[{}] {}\nURL: {}\n摘要: {}\n", i + 1, title, url, summary)
        }).collect();
        
        let output = format!("找到 {} 个搜索结果:\n\n{}", items.len(), formatted_results.join("\n"));
        
        info!("✅ IQS search completed with {} results", items.len());
        
        Ok(ToolResult {
            tool_call_id: tool_call.id.clone(),
            output,
            success: true,
        })
    } else {
        // 尝试兼容旧格式
        if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
            if results.is_empty() {
                return Ok(ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    output: "未找到相关搜索结果。".to_string(),
                    success: true,
                });
            }
            
            let formatted_results: Vec<String> = results.iter().enumerate().map(|(i, item)| {
                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("N/A");
                let url = item.get("url").or_else(|| item.get("link")).and_then(|u| u.as_str()).unwrap_or("N/A");
                let snippet = item.get("snippet").or_else(|| item.get("summary")).and_then(|s| s.as_str()).unwrap_or("N/A");
                
                format!("[{}] {}\nURL: {}\n摘要: {}\n", i + 1, title, url, snippet)
            }).collect();
            
            let output = format!("找到 {} 个搜索结果:\n\n{}", results.len(), formatted_results.join("\n"));
            
            info!("✅ IQS search completed with {} results (legacy format)", results.len());
            
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output,
                success: true,
            })
        } else {
            Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                output: format!("IQS API 返回格式异常，未找到 items 或 results 字段。完整响应: {}", result),
                success: false,
            })
        }
    }
}
