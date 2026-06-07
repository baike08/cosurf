/// React Agent Loop 引擎
/// 实现 ReAct (Reasoning + Acting) 模式的智能代理循环

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::ai::mcp::McpClient;
use crate::ai::sandbox::Sandbox;
use crate::error::{AppError, AppResult};

/// Agent 思考步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thought {
    pub reasoning: String,
    pub action: Option<Action>,
}

/// Agent 动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

/// Agent 观察结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub result: String,
    pub success: bool,
}

/// Agent 执行轨迹
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrace {
    pub thoughts: Vec<Thought>,
    pub observations: Vec<Observation>,
    pub final_answer: Option<String>,
}

/// Agent 配置
pub struct AgentConfig {
    pub max_iterations: usize,
    pub temperature: f64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            temperature: 0.7,
        }
    }
}

/// React Agent
pub struct ReactAgent {
    config: AgentConfig,
    mcp_client: McpClient,
    sandbox: Sandbox,
}

impl ReactAgent {
    pub fn new(config: AgentConfig, mcp_client: McpClient, sandbox: Sandbox) -> Self {
        Self {
            config,
            mcp_client,
            sandbox,
        }
    }

    /// 执行 Agent 循环
    pub async fn run(&mut self, task: &str) -> AppResult<AgentTrace> {
        info!(task = %task, "Starting React Agent loop");
        
        let mut trace = AgentTrace {
            thoughts: Vec::new(),
            observations: Vec::new(),
            final_answer: None,
        };

        // 构建初始提示
        let mut messages = vec![
            format!("You are a helpful AI assistant with access to various tools.\n\nTask: {}\n\nAvailable tools:", task),
        ];
        
        // 添加工具列表
        for tool in self.mcp_client.list_tools() {
            messages.push(format!("- {}: {}", tool.name, tool.description));
        }
        
        messages.push("\nLet's think step by step.".to_string());

        let mut iteration = 0;
        
        while iteration < self.config.max_iterations {
            iteration += 1;
            info!(iteration = iteration, "Agent iteration");

            // Step 1: LLM 思考
            let thought = self.llm_think(&messages.join("\n")).await?;
            trace.thoughts.push(thought.clone());

            // 如果没有动作,说明已经得出最终答案
            if thought.action.is_none() {
                trace.final_answer = Some(thought.reasoning);
                break;
            }

            let action = thought.action.unwrap();

            // Step 2: 执行动作
            let observation = self.execute_action(&action).await?;
            trace.observations.push(observation.clone());

            // Step 3: 更新消息历史
            messages.push(format!(
                "\nThought: {}\nAction: {}({})\nObservation: {}",
                thought.reasoning,
                action.tool_name,
                serde_json::to_string_pretty(&action.arguments)?,
                observation.result
            ));

            // 如果成功且不需要进一步操作,结束循环
            if observation.success && !observation.result.contains("need more") {
                // 再调用一次 LLM 获取最终答案
                let final_thought = self.llm_think(&messages.join("\n")).await?;
                trace.final_answer = Some(final_thought.reasoning);
                break;
            }
        }

        if trace.final_answer.is_none() {
            trace.final_answer = Some("Maximum iterations reached. Could not complete the task.".into());
        }

        info!(iterations = iteration, "Agent loop completed");
        Ok(trace)
    }

    /// LLM 思考步骤
    async fn llm_think(&self, context: &str) -> AppResult<Thought> {
        // TODO: 实际调用 LLM API
        // 这里先提供模拟实现
        
        info!("LLM thinking...");
        
        // 模拟 LLM 响应
        Ok(Thought {
            reasoning: "I need to gather more information to complete this task.".into(),
            action: None, // 简化:直接返回最终答案
        })
    }

    /// 执行动作
    async fn execute_action(&self, action: &Action) -> AppResult<Observation> {
        info!(tool = %action.tool_name, "Executing action");

        match action.tool_name.as_str() {
            // 内置工具
            "load_web_page" => {
                let url = action.arguments.get("url").and_then(|v| v.as_str()).unwrap_or("");
                match self.sandbox.load_web_page(url)? {
                    Some(content) => Ok(Observation {
                        result: format!("Loaded web page content ({} bytes)", content.len()),
                        success: true,
                    }),
                    None => Ok(Observation {
                        result: "Web page not found in cache".into(),
                        success: false,
                    }),
                }
            },
            "load_memory" => {
                let key = action.arguments.get("key").and_then(|v| v.as_str()).unwrap_or("");
                match self.sandbox.load_memory(key)? {
                    Some(value) => Ok(Observation {
                        result: value,
                        success: true,
                    }),
                    None => Ok(Observation {
                        result: format!("Memory '{}' not found", key),
                        success: false,
                    }),
                }
            },
            "search_memories" => {
                let query = action.arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
                let memories = self.sandbox.search_memories(query)?;
                Ok(Observation {
                    result: format!("Found {} memories", memories.len()),
                    success: true,
                })
            },
            "execute_command" => {
                let cmd = action.arguments.get("command").and_then(|v| v.as_str()).unwrap_or("");
                let args: Vec<&str> = action.arguments
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                
                match self.sandbox.execute_command(cmd, &args) {
                    Ok(output) => Ok(Observation {
                        result: output,
                        success: true,
                    }),
                    Err(e) => Ok(Observation {
                        result: format!("Error: {}", e),
                        success: false,
                    }),
                }
            },
            // MCP 工具
            _ => {
                match self.mcp_client.call_tool(&action.tool_name, &action.arguments).await {
                    Ok(result) => Ok(Observation {
                        result: serde_json::to_string_pretty(&result)?,
                        success: true,
                    }),
                    Err(e) => Ok(Observation {
                        result: format!("Error: {}", e),
                        success: false,
                    }),
                }
            }
        }
    }
}
