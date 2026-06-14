// 示例：使用优化版 Agent Loop
// 
// 这个文件展示了如何调用 stream_chat_completion_optimized 函数
// 以及智能并行调度和上下文管理的实际效果

use cosurf_native::ai::{
    provider::{ModelConfig, ChatMessage},
    stream::{stream_chat_completion_optimized, StreamCallbacks},
};

/// 创建示例对话消息
fn create_sample_messages() -> Vec<ChatMessage> {
    vec![
        // 系统提示词（会被冻结）
        ChatMessage {
            role: "system".to_string(),
            content: "你是一个智能助手，可以帮助用户完成各种任务。".to_string(),
            name: None,
            tool_call_id: None,
        },
        
        // 用户首问（会被冻结）
        ChatMessage {
            role: "user".to_string(),
            content: "请帮我搜索 Rust 编程语言的教程，总结关键点，然后导出为 Markdown 文件。".to_string(),
            name: None,
            tool_call_id: None,
        },
    ]
}

/// 模拟流式回调
fn create_sample_callbacks() -> StreamCallbacks {
    StreamCallbacks {
        on_chunk: Box::new(|conversation_id, message_id, chunk, is_thinking, is_done| {
            println!(
                "[Chunk] conv={}, msg={}, thinking={}, done={}: {}",
                conversation_id, message_id, is_thinking, is_done, chunk
            );
        }),
        on_tool_call_start: Box::new(|conversation_id, message_id, tool_call| {
            println!(
                "[Tool Start] {} - {} (id: {})",
                conversation_id, tool_call.name, tool_call.id
            );
        }),
        on_tool_result: Box::new(|conversation_id, message_id, tool_call, result| {
            println!(
                "[Tool Result] {} - {}: success={}, output_length={}",
                conversation_id,
                tool_call.name,
                result.success,
                result.output.len()
            );
        }),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 配置模型
    let config = ModelConfig {
        provider: "openai".to_string(),
        model: "gpt-4-turbo".to_string(),
        api_key: "sk-xxx".to_string(),
        base_url: None,
        temperature: Some(0.7),
        max_tokens: Some(4000),
    };
    
    // 2. 准备对话消息
    let messages = create_sample_messages();
    
    // 3. 创建回调
    let callbacks = create_sample_callbacks();
    
    // 4. Skills schemas（可选）
    let skills_schemas = vec![];
    
    // 5. 调用优化的 Agent Loop
    println!("🚀 Starting Optimized Agent Loop...");
    
    stream_chat_completion_optimized(
        &config,
        messages,
        "conv-123",
        "msg-456",
        &callbacks,
        skills_schemas,
    ).await?;
    
    println!("✅ Agent Loop completed!");
    
    Ok(())
}

/*
预期输出示例：

🚀 Starting Optimized Agent Loop...
🤖 Starting Optimized Agent Loop for conversation conv-123
📦 ContextManager initialized: frozen=2, compressible=0, tokens=45
🔄 Optimized Agent Loop iteration 1/30
[Chunk] conv=conv-123, msg=msg-456, thinking=false, done=false: 我需要执行以下工具调用...
🔧 Found 5 tool calls in iteration 1
📊 Smart scheduling: read=2, write=1, network=1, browser=0
📖 Executing 2 read tools in parallel
🌐 Executing 1 network tools in parallel
[Tool Start] conv-123 - summarize_page (id: tc-1)
[Tool Start] conv-123 - translate (id: tc-2)
[Tool Start] conv-123 - mcp_iqs_search (id: tc-3)
[Tool Result] conv-123 - summarize_page: success=true, output_length=1250
[Tool Result] conv-123 - translate: success=true, output_length=890
[Tool Result] conv-123 - mcp_iqs_search: success=true, output_length=1500
✍️  Executing write tool: export_markdown
[Tool Start] conv-123 - export_markdown (id: tc-5)
[Tool Result] conv-123 - export_markdown: success=true, output_length=45
❄️  Freezing 1 important messages
🔄 Optimized Agent Loop iteration 2/30
[Chunk] conv=conv-123, msg=msg-456, thinking=false, done=true: 我已经完成了所有任务...
✅ No more tool calls, agent loop completed after 2 iterations
🏁 Optimized Agent Loop completed after 2 iterations
✅ Agent Loop completed!

性能分析：
- 迭代 1: 5 个工具调用
  * Read 工具并行: max(1.2s, 0.9s) = 1.2s （节省 0.9s）
  * Network 工具并行: max(2.1s, 1.8s) = 2.1s （节省 1.8s）
  * Write 工具串行: 0.5s
  * 总耗时: 3.8s vs 6.5s（优化前） = 加速 1.71x

- 上下文管理:
  * 初始 Token: 45
  * 添加工具结果后: 45 + 5745 = 5790
  * 冻结重要消息: 1 个 MCP 工具结果被冻结
  * 未触发压缩（Token < 预算）
*/
