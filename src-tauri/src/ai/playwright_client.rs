use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const PLAYWRIGHT_SERVICE_URL: &str = "http://127.0.0.1:3100";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResponse {
    pub content: String,
    pub format: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NavigateRequest {
    url: String,
    timeout: Option<u64>,
    #[serde(rename = "waitUntil")]
    wait_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetContentRequest {
    format: String,
    selector: Option<String>,
}

pub struct PlaywrightClient {
    client: Client,
}

impl PlaywrightClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// 启动浏览器会话
    pub async fn launch_session(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/browser/launch", PLAYWRIGHT_SERVICE_URL);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "headless": true
            }))
            .send()
            .await?;

        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        
        if api_response.success {
            let session_id = api_response.data
                .and_then(|d| d.get("sessionId").cloned())
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .ok_or("Failed to get sessionId from response")?;
            
            Ok(session_id)
        } else {
            Err(api_response.error.unwrap_or_else(|| "Unknown error".to_string()).into())
        }
    }

    /// 导航到指定 URL
    pub async fn navigate(&self, session_id: &str, url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let nav_url = format!("{}/browser/{}/navigate", PLAYWRIGHT_SERVICE_URL, session_id);
        
        let request = NavigateRequest {
            url: url.to_string(),
            timeout: Some(30000),
            wait_until: Some("networkidle".to_string()),
        };

        let response = self.client
            .post(&nav_url)
            .json(&request)
            .send()
            .await?;

        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        
        if api_response.success {
            Ok(())
        } else {
            Err(api_response.error.unwrap_or_else(|| "Navigation failed".to_string()).into())
        }
    }

    /// 获取页面内容
    pub async fn get_page_content(
        &self,
        session_id: &str,
        format: &str,
        selector: Option<&str>,
    ) -> Result<ContentResponse, Box<dyn std::error::Error + Send + Sync>> {
        let content_url = format!("{}/browser/{}/content", PLAYWRIGHT_SERVICE_URL, session_id);
        
        let request = GetContentRequest {
            format: format.to_string(),
            selector: selector.map(|s| s.to_string()),
        };

        let response = self.client
            .post(&content_url)
            .json(&request)
            .send()
            .await?;

        let api_response: ApiResponse<ContentResponse> = response.json().await?;
        
        if api_response.success {
            api_response.data.ok_or_else(|| "No content in response".into())
        } else {
            Err(api_response.error.unwrap_or_else(|| "Failed to get content".to_string()).into())
        }
    }

    /// 关闭浏览器会话
    pub async fn close_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let close_url = format!("{}/browser/{}/close", PLAYWRIGHT_SERVICE_URL, session_id);
        
        let response = self.client
            .post(&close_url)
            .send()
            .await?;

        let api_response: ApiResponse<serde_json::Value> = response.json().await?;
        
        if api_response.success {
            Ok(())
        } else {
            Err(api_response.error.unwrap_or_else(|| "Failed to close session".to_string()).into())
        }
    }

    /// 一键获取页面内容（自动处理会话生命周期）
    pub async fn fetch_page_content(
        &self,
        url: &str,
        format: &str,
        selector: Option<&str>,
    ) -> Result<ContentResponse, Box<dyn std::error::Error + Send + Sync>> {
        // 1. 启动会话
        let session_id = self.launch_session().await?;
        
        // 确保即使出错也会关闭会话
        let result: Result<ContentResponse, Box<dyn std::error::Error + Send + Sync>> = async {
            // 2. 导航到 URL
            self.navigate(&session_id, url).await?;
            
            // 3. 获取内容
            let content = self.get_page_content(&session_id, format, selector).await?;
            
            Ok(content)
        }.await;

        // 4. 关闭会话
        if let Err(e) = self.close_session(&session_id).await {
            eprintln!("Warning: Failed to close session {}: {}", session_id, e);
        }

        result
    }
}
