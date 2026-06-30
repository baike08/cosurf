//! Codex Adapter Integration Tests (using mock CLI)
//!
//! These tests verify the Codex CLI integration works correctly.
//! Run with: CODEX_USE_MOCK=true cargo test --lib codex_adapter_integration

#[cfg(test)]
mod codex_adapter_integration {
    use crate::ai::codex_adapter::{CodexAgent, CodexAgentConfig};
    
    #[tokio::test]
    async fn test_codex_agent_with_mock() {
        // Set mock mode
        std::env::set_var("CODEX_USE_MOCK", "true");
        
        let config = CodexAgentConfig {
            provider: "mock".to_string(),
            model: "mock-model".to_string(),
            api_key: "mock-key".to_string(),
            base_url: None,
            cwd: "/tmp".to_string(),
            codex_home: "/tmp/codex".to_string(),
        };
        
        // Create agent
        let agent = CodexAgent::new(config).await;
        assert!(agent.is_ok(), "Failed to create Codex Agent: {:?}", agent.err());
        
        let agent = agent.unwrap();
        
        // Start thread
        let mut thread = agent.start_thread().await;
        assert!(thread.is_ok(), "Failed to start thread: {:?}", thread.err());
        
        let mut thread = thread.unwrap();
        
        println!("✅ Thread started: {}", thread.thread_id());
        
        // Send message and receive stream
        let stream = agent.send_message_stream(&mut thread, "Hello").await;
        assert!(stream.is_ok(), "Failed to send message: {:?}", stream.err());
        
        let mut rx = stream.unwrap();
        
        // Collect responses
        let mut responses = Vec::new();
        while let Some(text) = rx.recv().await {
            println!("📨 Received: {}", text);
            responses.push(text);
        }
        
        assert!(!responses.is_empty(), "No responses received");
        println!("✅ Received {} response chunks", responses.len());
        
        // Cleanup
        std::env::remove_var("CODEX_USE_MOCK");
    }
    
    #[tokio::test]
    async fn test_codex_agent_without_mock_should_fail() {
        // Ensure mock mode is off
        std::env::remove_var("CODEX_USE_MOCK");
        std::env::remove_var("CODEX_BINARY_PATH");
        
        let config = CodexAgentConfig {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_key: "sk-test".to_string(),
            base_url: None,
            cwd: "/tmp".to_string(),
            codex_home: "/tmp/codex".to_string(),
        };
        
        // This should fail because codex CLI is not installed
        let result = CodexAgent::new(config).await;
        assert!(result.is_err(), "Should fail without mock or real codex CLI");
        
        println!("✅ Correctly failed without codex CLI: {:?}", result.err());
    }
}
