use tauri::{AppHandle, Emitter, State};
use tracing::{error, info};

use crate::ai::provider::ChatMessage;
use crate::ai::stream::stream_chat_completion;
use crate::db::messages::{CreateMessageRequest, StreamChunk};
use crate::error::ErrorResponse;
use crate::state::AppState;

#[tauri::command]
pub fn stop_generation(state: State<'_, AppState>) -> Result<(), ErrorResponse> {
    state.cancel_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn send_chat_message(
    app: AppHandle,
    state: State<'_, AppState>,
    conversation_id: String,
    content: String,
) -> Result<StreamChunk, ErrorResponse> {
    info!("send_chat_message called: conv={}, content_len={}", conversation_id, content.len());

    let (model_config, _user_message, assistant_message_id) = {
        let db = state.db.lock().map_err(|e| ErrorResponse {
            code: "LOCK_ERROR".into(),
            message: e.to_string(),
        })?;

        let model_config = db
            .get_active_model_config()
            .map_err(|e| ErrorResponse::from(e))?
            .ok_or_else(|| ErrorResponse {
                code: "NO_MODEL".into(),
                message: "No active model configured. Please configure a model first.".into(),
            })?;

        // 打印模型配置信息用于调试
        info!("=== Model Config Debug ===");
        info!("Model ID: {}", model_config.model_id);
        info!("Provider: {}", model_config.provider);
        info!("Base URL: {:?}", model_config.base_url);
        info!("=========================");

        let user_msg = db
            .create_message(&CreateMessageRequest {
                conversation_id: conversation_id.clone(),
                role: "user".into(),
                content: content.clone(),
                attachments: vec![],
            })
            .map_err(|e| ErrorResponse::from(e))?;

        let _ = db
            .update_message(
                &user_msg.id,
                &crate::db::messages::UpdateMessageRequest {
                    content: None,
                    status: Some("complete".into()),
                },
            )
            .map_err(|e| ErrorResponse::from(e))?;

        let assistant_msg = db
            .create_message(&CreateMessageRequest {
                conversation_id: conversation_id.clone(),
                role: "assistant".into(),
                content: String::new(),
                attachments: vec![],
            })
            .map_err(|e| ErrorResponse::from(e))?;

        (model_config, user_msg, assistant_msg.id)
    };

    let chat_messages = {
        let db = state.db.lock().map_err(|e| ErrorResponse {
            code: "LOCK_ERROR".into(),
            message: e.to_string(),
        })?;

        // 获取用户名称设置
        let user_name = db.get_setting("userName")
            .ok()
            .flatten()
            .unwrap_or_else(|| "CoCo".to_string());

        let history = db
            .list_messages(&conversation_id)
            .map_err(|e| ErrorResponse::from(e))?;

        let mut msgs: Vec<ChatMessage> = history
            .into_iter()
            .filter(|m| m.id != assistant_message_id && m.status == "complete")
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content,
                name: None,
                tool_call_id: None,
            })
            .collect();

        // 在消息开头添加系统提示词（定义 AI 角色）
        let system_prompt = format!(
            r#"你是 CoSurf 智能助手，一个真诚、乐于助人的 AI 思考伙伴。

## CoSurf 功能特性
1. **智能浏览**：帮助用户浏览网页、提取关键信息、生成摘要
2. **AI 对话**：支持流式响应、思考过程展示、多轮对话记忆
3. **会话管理**：自动保存对话历史到 SQLite，重启后可查看
4. **截图工具**：快速截图，支持区域选择和复制/保存
5. **标签页管理**：多标签页浏览，状态保持不丢失
6. **隐私保护**：支持隐私模式，启用后不保存浏览历史

## 你的能力
- 解答各种问题、提供创意灵感
- 编写和调试代码、分析逻辑
- 结合当前浏览的网页内容进行智能回答
- 记住用户的行为偏好和关键信息
- 陪你聊天、提供情感支持

## 可用工具
你拥有以下工具来帮助用户完成任务，当用户请求涉及网页操作时，请主动使用这些工具：

1. **open_url** - 打开新的网页
   - 用途：当用户要求打开某个网站或URL时使用
   - 参数：url (字符串，必须以 http:// 或 https:// 开头)
   - 示例：用户说"打开百度" → 调用 open_url(url="https://www.baidu.com")

2. **summarize_page** - 总结当前页面内容
   - 用途：当用户要求总结、概括页面内容时使用
   - 参数：max_length (可选，整数，默认500)

3. **web_agent** - 在网页上执行自动化操作
   - 用途：点击按钮、填写表单、滚动页面等
   - 参数：action (click/fill/select/scroll/wait), selector (CSS选择器), value (可选)

4. **screenshot** - 截取当前页面
   - 用途：当用户需要了解页面视觉布局时使用
   - 参数：full_page (可选，布尔值)

5. **translate** - 翻译页面内容
   - 用途：将页面翻译成指定语言
   - 参数：target_language (如 'zh', 'en', 'ja')

6. **web_search** - 联网搜索（使用阿里云 IQS）
   - 用途：获取最新信息、实时数据、新闻热点
   - 参数：query (搜索词), engine_type (Generic/News/Academic), time_range (OneDay/OneWeek/OneMonth), max_results (1-20)
   - 注意：需要在设置中配置 ALIYUN_IQS_API_KEY 才能使用

7. **run_command** - 执行 shell 命令
   - 用途：在系统终端执行命令，运行脚本、调用 CLI 工具等
   - 参数：command (要执行的命令), working_dir (可选，工作目录), timeout (可选，超时秒数，默认 30)
   - 注意：Windows 上通过 cmd /C 执行，Linux/macOS 上通过 sh -c 执行
   - 示例：用户说"运行 python --version" → 调用 run_command(command="python --version")

## 交互风格
- 真诚、友好、乐于助人
- 回答清晰、结构化、易于理解
- 在需要时展示思考过程，让用户了解推理路径
- 根据上下文提供个性化建议
- **当用户请求涉及网页操作时，优先使用工具而不是仅口头回复**

请始终保持专业、友好的态度，为用户提供有价值的帮助。

## 重要提示
- 当前用户的名称是：{user_name}
- 在对话中请使用这个名称称呼用户"#,
            user_name = user_name
        );
        
        msgs.insert(0, ChatMessage {
            role: "system".into(),
            content: system_prompt,
            name: None,
            tool_call_id: None,
        });

        if msgs.is_empty() || msgs.last().map(|m| m.content.as_str()) != Some(&content) {
            msgs.push(ChatMessage {
                role: "user".into(),
                content,
                name: None,
                tool_call_id: None,
            });
        }

        // 输出完整的 prompt 用于调试
        info!("=== AI Prompt Debug ===");
        info!("Conversation ID: {}", conversation_id);
        info!("Total messages: {}", msgs.len());
        for (i, msg) in msgs.iter().enumerate() {
            let preview = if msg.content.chars().count() > 2000 {
                format!("{}...", msg.content.chars().take(2000).collect::<String>())
            } else {
                msg.content.clone()
            };
            info!("Message {}: [{}] {}", i, msg.role, preview);
        }
        info!("======================");

        msgs
    };

    let conv_id = conversation_id.clone();
    let msg_id = assistant_message_id.clone();
    let app_clone = app.clone();

    tokio::spawn(async move {
        let app_inner = app_clone.clone();
        let conv_id_inner = conv_id.clone();
        let msg_id_inner = msg_id.clone();
        
        // 使用 catch_unwind 捕获 panic
        let result = tokio::task::spawn(async move {
            stream_chat_completion(
                app_inner,
                &model_config,
                chat_messages,
                &conv_id_inner,
                &msg_id_inner,
            )
            .await
        }).await;

        match result {
            Ok(Ok(())) => {
                info!("AI stream completed successfully");
            }
            Ok(Err(e)) => {
                error!("AI streaming error: {}", e);
                let error_event = serde_json::json!({
                    "conversation_id": conv_id,
                    "error": format!("AI 服务错误: {}", e)
                });
                let _ = app_clone.emit("ai:stream-error", &error_event);
                let done_chunk = crate::db::messages::StreamChunk {
                    conversation_id: conv_id.clone(),
                    message_id: msg_id.clone(),
                    delta: String::new(),
                    is_thinking: false,
                    done: true,
                };
                let _ = app_clone.emit("ai:stream-chunk", &done_chunk);
            }
            Err(e) => {
                // tokio task panic
                error!("AI stream task panicked: {}", e);
                let error_event = serde_json::json!({
                    "conversation_id": conv_id,
                    "error": format!("AI 服务内部错误 (panic): {}", e)
                });
                let _ = app_clone.emit("ai:stream-error", &error_event);
                let done_chunk = crate::db::messages::StreamChunk {
                    conversation_id: conv_id.clone(),
                    message_id: msg_id.clone(),
                    delta: String::new(),
                    is_thinking: false,
                    done: true,
                };
                let _ = app_clone.emit("ai:stream-chunk", &done_chunk);
            }
        }
    });

    Ok(StreamChunk {
        conversation_id,
        message_id: assistant_message_id,
        delta: String::new(),
        is_thinking: false,
        done: false,
    })
}

#[tauri::command]
pub fn append_stream_chunk(
    state: State<'_, AppState>,
    message_id: String,
    delta: String,
    is_thinking: bool,
) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.append_message_content(&message_id, &delta, is_thinking)
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub fn complete_stream(state: State<'_, AppState>, message_id: String) -> Result<(), ErrorResponse> {
    let db = state.db.lock().map_err(|e| ErrorResponse {
        code: "LOCK_ERROR".into(),
        message: e.to_string(),
    })?;
    db.complete_message(&message_id)
        .map_err(|e| ErrorResponse::from(e))
}

#[tauri::command]
pub async fn generate_conversation_title(
    app: AppHandle,
    state: State<'_, AppState>,
    context: String,
) -> Result<String, ErrorResponse> {
    info!("generate_conversation_title called: context_len={}", context.len());

    let model_config = {
        let db = state.db.lock().map_err(|e| ErrorResponse {
            code: "LOCK_ERROR".into(),
            message: e.to_string(),
        })?;

        db.get_active_model_config()
            .map_err(|e| ErrorResponse::from(e))?
            .ok_or_else(|| ErrorResponse {
                code: "NO_MODEL".into(),
                message: "No active model configured. Please configure a model first.".into(),
            })?
    };

    // 构建生成标题的系统提示词
    let system_prompt = r#"你是一个专业的对话标题生成器。请根据提供的对话内容，生成一个简洁、准确的标题（不超过15个字）。

要求：
- 只返回标题文本，不要包含任何解释或额外内容
- 标题应该概括对话的核心主题
- 使用中文
- 保持简洁明了"#
    ;

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: system_prompt.to_string(),
            name: None,
            tool_call_id: None,
        },
        ChatMessage {
            role: "user".into(),
            content: format!("请为以下对话生成一个标题：\n\n{}", context),
            name: None,
            tool_call_id: None,
        },
    ];

    // 使用 reqwest 直接调用 API（非流式）
    use reqwest::Client;
    use crate::ai::provider::{build_chat_request, build_headers, get_api_url, ChatResponse};
    
    let url = get_api_url(&model_config).map_err(|e| ErrorResponse {
        code: "CONFIG_ERROR".into(),
        message: e.to_string(),
    })?;
    
    let headers = build_headers(&model_config).map_err(|e| ErrorResponse {
        code: "CONFIG_ERROR".into(),
        message: e.to_string(),
    })?;
    
    // 构建非流式请求
    let request_body = build_chat_request(&model_config, messages);
    let mut non_stream_request = request_body.clone();
    non_stream_request.stream = false;
    
    let client = Client::new();
    let mut request = client.post(&url);
    
    for (key, value) in &headers {
        request = request.header(key.as_str(), value.as_str());
    }
    
    let body_json = serde_json::to_string(&non_stream_request).map_err(|e| ErrorResponse {
        code: "SERIALIZE_ERROR".into(),
        message: e.to_string(),
    })?;
    
    let response = request.body(body_json).send().await.map_err(|e| ErrorResponse {
        code: "NETWORK_ERROR".into(),
        message: e.to_string(),
    })?;
    
    let chat_response: ChatResponse = response.json().await.map_err(|e| ErrorResponse {
        code: "PARSE_ERROR".into(),
        message: e.to_string(),
    })?;
    
    // 提取标题
    let title = chat_response.choices.first()
        .map(|c| c.message.content.trim().to_string())
        .unwrap_or_else(|| "新对话".to_string());
    
    info!("Generated conversation title: {}", title);
    Ok(title)
}
