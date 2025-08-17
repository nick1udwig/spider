use hyperprocess_macro::*;
use hyperware_process_lib::{
    our,
    homepage::add_to_homepage,
};
use hyperware_app_common::SaveOptions;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;

#[derive(Default, Serialize, Deserialize)]
pub struct SpiderState {
    api_keys: Vec<(String, ApiKey)>,
    spider_api_keys: Vec<SpiderApiKey>,
    mcp_servers: Vec<McpServer>,
    active_conversations: Vec<(String, Conversation)>,
    default_llm_provider: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ApiKey {
    provider: String,
    key: String,
    created_at: u64,
    last_used: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SpiderApiKey {
    key: String,
    name: String,
    permissions: Vec<String>,
    created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct McpServer {
    id: String,
    name: String,
    transport: TransportConfig,
    tools: Vec<Tool>,
    connected: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct TransportConfig {
    transport_type: String, // "stdio" or "http"
    command: Option<String>,
    args: Option<Vec<String>>,
    url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Tool {
    name: String,
    description: String,
    parameters: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Conversation {
    id: String,
    messages: Vec<Message>,
    metadata: ConversationMetadata,
    llm_provider: String,
    mcp_servers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ConversationMetadata {
    start_time: String,
    client: String,
    from_stt: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Message {
    role: String,
    content: String,
    tool_calls_json: Option<String>, // JSON string of tool calls
    tool_results_json: Option<String>, // JSON string of tool results
    timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ToolCall {
    id: String,
    tool_name: String,
    parameters: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ToolResult {
    tool_call_id: String,
    result: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct SetApiKeyRequest {
    provider: String,
    key: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CreateSpiderKeyRequest {
    name: String,
    permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct AddMcpServerRequest {
    name: String,
    transport: TransportConfig,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ListConversationsRequest {
    limit: Option<u32>,
    offset: Option<u32>,
    client: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct UpdateConfigRequest {
    default_llm_provider: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ChatRequest {
    api_key: String,
    messages: Vec<Message>,
    llm_provider: Option<String>,
    mcp_servers: Option<Vec<String>>,
    metadata: Option<ConversationMetadata>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ChatResponse {
    conversation_id: String,
    response: Message,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ProcessRequest {
    action: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ProcessResponse {
    success: bool,
    data: String,
}

#[hyperprocess(
    name = "Spider",
    ui = Some(HttpBindingConfig::default()),
    endpoints = vec![
        Binding::Http { 
            path: "/api", 
            config: HttpBindingConfig::new(false, false, false, None) 
        }
    ],
    save_config = SaveOptions::OnDiff,
    wit_world = "spider-dot-os-v0"
)]
impl SpiderState {
    #[init]
    async fn initialize(&mut self) {
        add_to_homepage("Spider", Some("🕷️"), Some("/"), None);
        
        self.default_llm_provider = "anthropic".to_string();
        self.max_tokens = 4096;
        self.temperature = 0.7;
        
        let our_node = our().node.clone();
        println!("Spider MCP client initialized on node: {}", our_node);
        
        // VFS directory creation will be handled when actually saving files
    }
    
    #[http]
    async fn set_api_key(&mut self, request: SetApiKeyRequest) -> Result<String, String> {
        let encrypted_key = self.encrypt_key(&request.key);
        
        let api_key = ApiKey {
            provider: request.provider.clone(),
            key: encrypted_key,
            created_at: Utc::now().timestamp() as u64,
            last_used: None,
        };
        
        self.api_keys.retain(|(p, _)| p != &request.provider);
        self.api_keys.push((request.provider.clone(), api_key));
        
        Ok(format!("API key for {} set successfully", request.provider))
    }
    
    #[http]
    async fn list_api_keys(&self) -> String {
        let keys: Vec<_> = self.api_keys.iter().map(|(provider, key)| {
            serde_json::json!({
                "provider": provider,
                "created_at": key.created_at,
                "last_used": key.last_used,
                "key_preview": self.preview_key(&key.key),
            })
        }).collect();
        
        serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string())
    }
    
    #[http]
    async fn remove_api_key(&mut self, provider: String) -> Result<String, String> {
        let initial_len = self.api_keys.len();
        self.api_keys.retain(|(p, _)| p != &provider);
        
        if self.api_keys.len() < initial_len {
            Ok(format!("API key for {} removed", provider))
        } else {
            Err(format!("No API key found for {}", provider))
        }
    }
    
    #[http]
    async fn create_spider_key(&mut self, request: CreateSpiderKeyRequest) -> Result<String, String> {
        let key = format!("sp_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        let spider_key = SpiderApiKey {
            key: key.clone(),
            name: request.name,
            permissions: request.permissions,
            created_at: Utc::now().timestamp() as u64,
        };
        
        self.spider_api_keys.push(spider_key.clone());
        
        Ok(serde_json::to_string(&spider_key).unwrap())
    }
    
    #[http]
    async fn list_spider_keys(&self) -> String {
        serde_json::to_string(&self.spider_api_keys).unwrap_or_else(|_| "[]".to_string())
    }
    
    #[http]
    async fn revoke_spider_key(&mut self, key_id: String) -> Result<String, String> {
        let initial_len = self.spider_api_keys.len();
        self.spider_api_keys.retain(|k| k.key != key_id);
        
        if self.spider_api_keys.len() < initial_len {
            Ok(format!("Spider API key {} revoked", key_id))
        } else {
            Err(format!("Spider API key {} not found", key_id))
        }
    }
    
    #[http]
    async fn add_mcp_server(&mut self, request: AddMcpServerRequest) -> Result<String, String> {
        let server = McpServer {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            transport: request.transport,
            tools: Vec::new(),
            connected: false,
        };
        
        let server_id = server.id.clone();
        self.mcp_servers.push(server);
        
        Ok(server_id)
    }
    
    #[http]
    async fn list_mcp_servers(&self) -> String {
        serde_json::to_string(&self.mcp_servers).unwrap_or_else(|_| "[]".to_string())
    }
    
    #[http]
    async fn connect_mcp_server(&mut self, server_id: String) -> Result<String, String> {
        // Find the server and get its transport config
        let (server_name, transport) = {
            let server = self.mcp_servers.iter()
                .find(|s| s.id == server_id)
                .ok_or_else(|| format!("MCP server {} not found", server_id))?;
            (server.name.clone(), server.transport.clone())
        };
        
        // Discover tools from the MCP server
        let tools = self.discover_mcp_tools(&transport).await?;
        let tool_count = tools.len();
        
        // Update the server with discovered tools
        if let Some(server) = self.mcp_servers.iter_mut().find(|s| s.id == server_id) {
            server.tools = tools;
            server.connected = true;
        }
        
        Ok(format!("Connected to MCP server {} with {} tools", server_name, tool_count))
    }
    
    #[http]
    async fn list_conversations(&self, request: ListConversationsRequest) -> String {
        let conversations: Vec<_> = self.active_conversations.iter()
            .filter(|(_, conv)| {
                request.client.as_ref().map_or(true, |c| &conv.metadata.client == c)
            })
            .map(|(_, conv)| conv.clone())
            .skip(request.offset.unwrap_or(0) as usize)
            .take(request.limit.unwrap_or(50) as usize)
            .collect();
        
        serde_json::to_string(&conversations).unwrap_or_else(|_| "[]".to_string())
    }
    
    #[http]
    async fn get_conversation(&self, conversation_id: String) -> String {
        // First check in-memory conversations
        for (id, conv) in &self.active_conversations {
            if id == &conversation_id {
                return serde_json::to_string(conv).unwrap_or_else(|_| "{}".to_string());
            }
        }
        
        // Try to load from VFS
        match self.load_conversation_from_vfs(&conversation_id).await {
            Ok(conv) => serde_json::to_string(&conv).unwrap_or_else(|_| "{}".to_string()),
            Err(_) => "{}".to_string()
        }
    }
    
    #[http]
    async fn get_config(&self) -> String {
        serde_json::json!({
            "default_llm_provider": self.default_llm_provider,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
        }).to_string()
    }
    
    #[http]
    async fn update_config(&mut self, request: UpdateConfigRequest) -> Result<String, String> {
        if let Some(provider) = request.default_llm_provider {
            self.default_llm_provider = provider;
        }
        
        if let Some(tokens) = request.max_tokens {
            self.max_tokens = tokens;
        }
        
        if let Some(temp) = request.temperature {
            self.temperature = temp;
        }
        
        Ok("Configuration updated".to_string())
    }
    
    #[http]
    async fn chat(&mut self, request: ChatRequest) -> Result<String, String> {
        if !self.validate_spider_key(&request.api_key) {
            return Err("Invalid Spider API key".to_string());
        }
        
        let conversation_id = Uuid::new_v4().to_string();
        let llm_provider = request.llm_provider.unwrap_or(self.default_llm_provider.clone());
        
        println!("Spider: Starting new conversation {} with provider {}", conversation_id, llm_provider);
        
        // Get the API key for the selected provider
        let api_key = self.api_keys.iter()
            .find(|(p, _)| p == &llm_provider)
            .map(|(_, k)| k.key.clone())
            .ok_or_else(|| format!("No API key found for provider: {}", llm_provider))?;
        
        // Collect available tools from connected MCP servers
        let available_tools: Vec<Tool> = if let Some(ref mcp_server_ids) = request.mcp_servers {
            self.mcp_servers.iter()
                .filter(|s| s.connected && mcp_server_ids.contains(&s.id))
                .flat_map(|s| s.tools.clone())
                .collect()
        } else {
            // Use all connected servers if none specified
            self.mcp_servers.iter()
                .filter(|s| s.connected)
                .flat_map(|s| s.tools.clone())
                .collect()
        };
        
        // Start the agentic loop - runs indefinitely until the agent stops making tool calls
        let mut working_messages = request.messages.clone();
        let mut iteration_count = 0;
        
        let response = loop {
            iteration_count += 1;
            
            // Call the LLM with available tools
            let llm_response = if llm_provider == "anthropic" {
                self.call_anthropic_api(&api_key, &working_messages, &available_tools).await?
            } else {
                return Err(format!("Unsupported LLM provider: {}", llm_provider));
            };
            
            // Check if the response contains tool calls
            if let Some(ref tool_calls_json) = llm_response.tool_calls_json {
                // The agent wants to use tools - execute them
                println!("Spider: Iteration {} - Agent requested tool calls", iteration_count);
                let tool_results = self.process_tool_calls(tool_calls_json).await?;
                
                // Add the assistant's message with tool calls
                working_messages.push(llm_response.clone());
                
                // Add tool results as a new message for the LLM to see
                working_messages.push(Message {
                    role: "tool".to_string(),
                    content: "Tool execution results".to_string(),
                    tool_calls_json: None,
                    tool_results_json: Some(serde_json::to_string(&tool_results).unwrap()),
                    timestamp: Utc::now().timestamp() as u64,
                });
                
                // Continue the loop - the agent will decide what to do next
                continue;
            } else {
                // No tool calls - the agent has decided to provide a final response
                // Break the loop and return this response
                println!("Spider: Iteration {} - Agent provided final response (no tool calls)", iteration_count);
                break llm_response;
            }
        };
        
        // Add the final response to messages
        working_messages.push(response.clone());
        
        let metadata = request.metadata.unwrap_or(ConversationMetadata {
            start_time: Utc::now().to_rfc3339(),
            client: "unknown".to_string(),
            from_stt: false,
        });
        
        let conversation = Conversation {
            id: conversation_id.clone(),
            messages: working_messages,
            metadata,
            llm_provider,
            mcp_servers: request.mcp_servers.unwrap_or_default(),
        };
        
        // Save to VFS
        if let Err(e) = self.save_conversation_to_vfs(&conversation).await {
            println!("Warning: Failed to save conversation to VFS: {}", e);
        }
        
        // Keep in memory for quick access
        self.active_conversations.push((conversation_id.clone(), conversation));
        
        let chat_response = ChatResponse {
            conversation_id,
            response,
        };
        
        Ok(serde_json::to_string(&chat_response).unwrap())
    }
    
    #[local]
    async fn process_request(&mut self, request: ProcessRequest) -> Result<ProcessResponse, String> {
        match request.action.as_str() {
            "chat" => {
                let chat_request: ChatRequest = serde_json::from_str(&request.payload)
                    .map_err(|e| format!("Invalid chat request: {}", e))?;
                let result = self.chat(chat_request).await?;
                Ok(ProcessResponse {
                    success: true,
                    data: result,
                })
            }
            _ => {
                Ok(ProcessResponse {
                    success: false,
                    data: format!("Unknown action: {}", request.action),
                })
            }
        }
    }
}

impl SpiderState {
    fn encrypt_key(&self, key: &str) -> String {
        use sha2::{Sha256, Digest};
        use base64::{Engine as _, engine::general_purpose};
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        general_purpose::STANDARD.encode(result)
    }
    
    fn preview_key(&self, encrypted_key: &str) -> String {
        if encrypted_key.len() > 8 {
            format!("{}...", &encrypted_key[..8])
        } else {
            "***".to_string()
        }
    }
    
    fn validate_spider_key(&self, key: &str) -> bool {
        self.spider_api_keys.iter().any(|k| k.key == key)
    }
    
    async fn call_anthropic_api(&self, _api_key: &str, messages: &[Message], tools: &[Tool]) -> Result<Message, String> {
        #[derive(Serialize)]
        struct AnthropicRequest {
            model: String,
            messages: Vec<AnthropicMessage>,
            max_tokens: u32,
            temperature: f32,
            tools: Vec<AnthropicTool>,
        }
        
        #[derive(Serialize)]
        struct AnthropicMessage {
            role: String,
            content: String,
        }
        
        #[derive(Serialize)]
        struct AnthropicTool {
            name: String,
            description: String,
            input_schema: Value,
        }
        
        #[derive(Deserialize)]
        struct AnthropicResponse {
            content: Vec<AnthropicContent>,
        }
        
        #[derive(Deserialize)]
        struct AnthropicContent {
            text: String,
            tool_use: Option<AnthropicToolUse>,
        }
        
        #[derive(Deserialize)]
        struct AnthropicToolUse {
            id: String,
            name: String,
            input: Value,
        }
        
        // Convert our messages to Anthropic format
        let anthropic_messages: Vec<AnthropicMessage> = messages.iter()
            .filter(|m| m.role != "system")
            .map(|m| AnthropicMessage {
                role: if m.role == "assistant" { "assistant".to_string() } else { "user".to_string() },
                content: m.content.clone(),
            })
            .collect();
        
        // Convert tools to Anthropic format
        let anthropic_tools: Vec<AnthropicTool> = tools.iter()
            .map(|t| AnthropicTool {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: serde_json::from_str(&t.parameters).unwrap_or(Value::Object(serde_json::Map::new())),
            })
            .collect();
        
        let _request_body = AnthropicRequest {
            model: "claude-3-opus-20240229".to_string(),
            messages: anthropic_messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            tools: anthropic_tools,
        };
        
        // Simulate tool calling for demonstration
        // In a real implementation, this would come from the actual API response
        let simulate_tool_call = !tools.is_empty() && messages.len() % 3 == 1; // Simple heuristic for demo
        
        if simulate_tool_call && !tools.is_empty() {
            // Simulate a tool call
            let tool_calls = vec![
                ToolCall {
                    id: Uuid::new_v4().to_string(),
                    tool_name: tools[0].name.clone(),
                    parameters: r#"{"input":"test parameter"}"#.to_string(),
                }
            ];
            
            Ok(Message {
                role: "assistant".to_string(),
                content: "I'll help you with that. Let me use a tool to get the information.".to_string(),
                tool_calls_json: Some(serde_json::to_string(&tool_calls).unwrap()),
                tool_results_json: None,
                timestamp: Utc::now().timestamp() as u64,
            })
        } else {
            // Regular response
            Ok(Message {
                role: "assistant".to_string(),
                content: "Based on the available information, here's my response (placeholder - actual API integration pending)".to_string(),
                tool_calls_json: None,
                tool_results_json: None,
                timestamp: Utc::now().timestamp() as u64,
            })
        }
    }
    
    async fn save_conversation_to_vfs(&self, conversation: &Conversation) -> Result<(), String> {
        // For now, just store in memory
        // VFS integration requires proper PackageId setup which needs more configuration
        println!("Conversation {} saved to memory (VFS integration pending)", conversation.id);
        Ok(())
    }
    
    async fn load_conversation_from_vfs(&self, conversation_id: &str) -> Result<Conversation, String> {
        // For now, return error as VFS is not yet integrated
        Err(format!("Conversation {} not found (VFS integration pending)", conversation_id))
    }
    
    async fn discover_mcp_tools(&self, transport: &TransportConfig) -> Result<Vec<Tool>, String> {
        // MCP tool discovery implementation
        match transport.transport_type.as_str() {
            "stdio" => {
                // For stdio transport, we would spawn the process and communicate via stdin/stdout
                // This is a placeholder implementation
                Ok(vec![
                    Tool {
                        name: "example_tool".to_string(),
                        description: "An example tool from MCP server".to_string(),
                        parameters: r#"{"type":"object","properties":{"input":{"type":"string"}}}"#.to_string(),
                    }
                ])
            }
            "http" => {
                // For HTTP transport, we would make HTTP requests to discover tools
                // This is a placeholder implementation
                Ok(vec![
                    Tool {
                        name: "http_tool".to_string(),
                        description: "An HTTP-based MCP tool".to_string(),
                        parameters: r#"{"type":"object","properties":{"query":{"type":"string"}}}"#.to_string(),
                    }
                ])
            }
            _ => Err(format!("Unsupported transport type: {}", transport.transport_type))
        }
    }
    
    async fn execute_mcp_tool(&self, server_id: &str, tool_name: &str, parameters: &Value) -> Result<Value, String> {
        let server = self.mcp_servers.iter()
            .find(|s| s.id == server_id && s.connected)
            .ok_or_else(|| format!("MCP server {} not found or not connected", server_id))?;
        
        // Check if the tool exists
        let _tool = server.tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| format!("Tool {} not found on server {}", tool_name, server_id))?;
        
        // Execute the tool based on transport type
        match server.transport.transport_type.as_str() {
            "stdio" => {
                // Execute via stdio
                // This is a placeholder - actual implementation would spawn process and communicate
                Ok(serde_json::json!({
                    "result": format!("Executed {} with params: {}", tool_name, parameters),
                    "success": true
                }))
            }
            "http" => {
                // Execute via HTTP
                // This is a placeholder - actual implementation would make HTTP requests
                Ok(serde_json::json!({
                    "result": format!("HTTP execution of {} with params: {}", tool_name, parameters),
                    "success": true
                }))
            }
            _ => Err(format!("Unsupported transport type: {}", server.transport.transport_type))
        }
    }
    
    async fn process_tool_calls(&mut self, tool_calls_json: &str) -> Result<Vec<ToolResult>, String> {
        let tool_calls: Vec<ToolCall> = serde_json::from_str(tool_calls_json)
            .map_err(|e| format!("Failed to parse tool calls: {}", e))?;
        
        let mut results = Vec::new();
        
        for tool_call in tool_calls {
            // Find which MCP server has this tool
            let server = self.mcp_servers.iter()
                .find(|s| s.connected && s.tools.iter().any(|t| t.name == tool_call.tool_name));
            
            let result = if let Some(server) = server {
                let params: Value = serde_json::from_str(&tool_call.parameters)
                    .unwrap_or(Value::Object(serde_json::Map::new()));
                
                match self.execute_mcp_tool(&server.id, &tool_call.tool_name, &params).await {
                    Ok(res) => res.to_string(),
                    Err(e) => format!(r#"{{"error":"{}"}}"#, e)
                }
            } else {
                format!(r#"{{"error":"Tool {} not found in any connected MCP server"}}"#, tool_call.tool_name)
            };
            
            results.push(ToolResult {
                tool_call_id: tool_call.id,
                result,
            });
        }
        
        Ok(results)
    }
}