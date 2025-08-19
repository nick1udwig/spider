use hyperprocess_macro::*;
use hyperware_process_lib::{
    our,
    println,
    homepage::add_to_homepage,
    vfs::{open_dir, open_file, create_drive},
    LazyLoadBlob,
    http::{
        client::{open_ws_connection, send_ws_client_push},
        server::{WsMessageType, send_ws_push},
    },
};
use hyperware_anthropic_sdk::{
    AnthropicClient, CreateMessageRequest, Message as SdkMessage,
    Role, Content, ResponseContentBlock, Tool as SdkTool,
    ToolChoice
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
pub struct SpiderState {
    api_keys: Vec<(String, ApiKey)>,
    spider_api_keys: Vec<SpiderApiKey>,
    mcp_servers: Vec<McpServer>,
    active_conversations: Vec<(String, Conversation)>,
    default_llm_provider: String,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip)]
    ws_connections: HashMap<u32, WsConnection>,  // channel_id -> connection info
    #[serde(skip)]
    pending_mcp_requests: HashMap<String, PendingMcpRequest>, // request_id -> pending request
    #[serde(skip)]
    next_channel_id: u32,
    #[serde(skip)]
    chat_clients: HashMap<u32, ChatClient>,  // channel_id -> chat client connection
}

#[derive(Clone, Debug)]
struct WsConnection {
    server_id: String,
    server_name: String,
    channel_id: u32,
    tools: Vec<Tool>,
    initialized: bool,
}

#[derive(Clone, Debug)]
struct PendingMcpRequest {
    request_id: String,
    conversation_id: Option<String>,
    server_id: String,
    request_type: McpRequestType,
}

#[derive(Clone, Debug)]
enum McpRequestType {
    Initialize,
    ToolsList,
    ToolCall { tool_name: String },
}

#[derive(Clone, Debug)]
struct ChatClient {
    channel_id: u32,
    api_key: String,
    conversation_id: Option<String>,
    connected_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ApiKey {
    provider: String,
    key: String,
    created_at: u64,
    last_used: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ApiKeyInfo {
    provider: String,
    #[serde(rename = "createdAt")]
    created_at: u64,
    #[serde(rename = "lastUsed")]
    last_used: Option<u64>,
    #[serde(rename = "keyPreview")]
    key_preview: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SpiderApiKey {
    key: String,
    name: String,
    permissions: Vec<String>,
    #[serde(rename = "createdAt")]
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
    #[serde(rename = "transportType")]
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
    #[serde(rename = "llmProvider")]
    llm_provider: String,
    #[serde(rename = "mcpServers")]
    mcp_servers: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct ConversationMetadata {
    #[serde(rename = "startTime")]
    start_time: String,
    client: String,
    #[serde(rename = "fromStt")]
    from_stt: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Message {
    role: String,
    content: String,
    #[serde(rename = "toolCallsJson")]
    tool_calls_json: Option<String>, // JSON string of tool calls
    #[serde(rename = "toolResultsJson")]
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
    #[serde(rename = "defaultLlmProvider")]
    default_llm_provider: Option<String>,
    #[serde(rename = "maxTokens")]
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ChatRequest {
    #[serde(rename = "apiKey")]
    api_key: String,
    messages: Vec<Message>,
    #[serde(rename = "llmProvider")]
    llm_provider: Option<String>,
    #[serde(rename = "mcpServers")]
    mcp_servers: Option<Vec<String>>,
    metadata: Option<ConversationMetadata>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ChatResponse {
    #[serde(rename = "conversationId")]
    conversation_id: String,
    response: Message,
    #[serde(rename = "allMessages")]
    all_messages: Vec<Message>,  // Include all messages from the conversation
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ConfigResponse {
    #[serde(rename = "defaultLlmProvider")]
    default_llm_provider: String,
    #[serde(rename = "maxTokens")]
    max_tokens: u32,
    temperature: f32,
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
    save_config = hyperware_process_lib::hyperapp::SaveOptions::OnDiff,
    wit_world = "spider-dot-os-v0"
)]
impl SpiderState {
    #[init]
    async fn initialize(&mut self) {
        add_to_homepage("Spider", Some("🕷️"), Some("/"), None);

        self.default_llm_provider = "anthropic".to_string();
        self.max_tokens = 4096;
        self.temperature = 0.7;
        self.next_channel_id = 1000; // Start channel IDs at 1000

        let our_node = our().node.clone();
        println!("Spider MCP client initialized on node: {}", our_node);

        // Create an admin Spider key for the GUI
        let admin_key = SpiderApiKey {
            key: "sp_admin_gui_key".to_string(),
            name: "Admin GUI Key".to_string(),
            permissions: vec!["chat".to_string(), "read".to_string(), "write".to_string(), "admin".to_string()],
            created_at: Utc::now().timestamp() as u64,
        };

        // Check if admin key already exists
        if !self.spider_api_keys.iter().any(|k| k.key == admin_key.key) {
            self.spider_api_keys.push(admin_key.clone());
            println!("Spider: Created admin GUI key: {}", admin_key.key);
        }

        // VFS directory creation will be handled when actually saving files

        // Auto-reconnect to MCP servers that exist in state
        // Note: Don't filter by server.connected since they won't be connected on startup
        let servers_to_reconnect: Vec<McpServer> = self.mcp_servers
            .iter()
            .cloned()
            .collect();

        for server in servers_to_reconnect {
            println!("Auto-reconnecting to MCP server: {}", server.id);

            // Connect based on transport type
            if server.transport.transport_type == "websocket" || server.transport.transport_type == "stdio" {
                // Get WebSocket URL
                let ws_url = server.transport.url.clone()
                    .unwrap_or_else(|| "ws://localhost:10125".to_string());

                // Generate a new channel ID for this connection
                let channel_id = self.next_channel_id;
                self.next_channel_id += 1;

                // Open WebSocket connection
                match open_ws_connection(ws_url.clone(), None, channel_id).await {
                    Ok(_) => {
                        println!("Successfully initiated WebSocket connection to MCP server {}", server.id);

                        // Store connection info
                        self.ws_connections.insert(channel_id, WsConnection {
                            server_id: server.id.clone(),
                            server_name: server.name.clone(),
                            channel_id,
                            tools: Vec::new(),
                            initialized: false,
                        });

                        // Send initialize request
                        let init_request = serde_json::json!({
                            "jsonrpc": "2.0",
                            "method": "initialize",
                            "params": {
                                "protocolVersion": "2024-11-05",
                                "clientInfo": {
                                    "name": "spider",
                                    "version": "1.0.0"
                                },
                                "capabilities": {}
                            },
                            "id": format!("init_{}", channel_id)
                        });

                        // Store pending request
                        self.pending_mcp_requests.insert(
                            format!("init_{}", channel_id),
                            PendingMcpRequest {
                                request_id: format!("init_{}", channel_id),
                                conversation_id: None,
                                server_id: server.id.clone(),
                                request_type: McpRequestType::Initialize,
                            }
                        );

                        // Send the initialize request via WebSocket
                        let message = LazyLoadBlob::new(
                            Some("application/json".to_string()),
                            init_request.to_string().into_bytes()
                        );

                        send_ws_push(channel_id, WsMessageType::Text, message);
                        println!("Sent initialize request to MCP server {} (channel {})", server.id, channel_id);
                    }
                    Err(e) => {
                        println!("Failed to auto-reconnect to MCP server {}: {:?}", server.id, e);
                        if let Some(s) = self.mcp_servers.iter_mut().find(|s| s.id == server.id) {
                            s.connected = false;
                        }
                    }
                }
            } else {
                println!("Unsupported transport type for auto-reconnect: {}", server.transport.transport_type);
            }
        }
    }

    #[ws]
    async fn handle_websocket(&mut self, channel_id: u32, message_type: WsMessageType, blob: LazyLoadBlob) {
        // Handle WebSocket messages - could be either chat client or MCP server
        // Determine by checking if it's an existing MCP connection

        if self.ws_connections.contains_key(&channel_id) {
            // Handle as MCP server connection
            match message_type {
                WsMessageType::Text | WsMessageType::Binary => {
                    let message_bytes = blob.bytes.clone();
                    let message_str = String::from_utf8(message_bytes).unwrap_or_default();
                    if let Ok(json_msg) = serde_json::from_str::<Value>(&message_str) {
                        self.handle_mcp_message(channel_id, json_msg);
                    }
                }
                WsMessageType::Close => {
                    // Handle disconnection
                    if let Some(conn) = self.ws_connections.remove(&channel_id) {
                        println!("MCP server {} disconnected", conn.server_name);
                        // Update server status
                        if let Some(server) = self.mcp_servers.iter_mut().find(|s| s.id == conn.server_id) {
                            server.connected = false;
                        }
                    }
                }
                WsMessageType::Ping | WsMessageType::Pong => {
                    // Ignore ping/pong
                }
            }
        } else {
            // Handle as potential chat client
            match message_type {
                WsMessageType::Text | WsMessageType::Binary => {
                    let message_bytes = blob.bytes.clone();
                    let message_str = String::from_utf8(message_bytes).unwrap_or_default();

                    // Parse the incoming message
                    if let Ok(json_msg) = serde_json::from_str::<Value>(&message_str) {
                        // Handle different message types
                        if let Some(msg_type) = json_msg.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "auth" => {
                                    // Authenticate the client
                                    if let Some(api_key) = json_msg.get("apiKey").and_then(|v| v.as_str()) {
                                        if self.validate_spider_key(api_key) {
                                            self.chat_clients.insert(channel_id, ChatClient {
                                                channel_id,
                                                api_key: api_key.to_string(),
                                                conversation_id: None,
                                                connected_at: Utc::now().timestamp() as u64,
                                            });

                                            // Send auth success response
                                            let response = serde_json::json!({
                                                "type": "auth_success",
                                                "message": "Authenticated successfully"
                                            });
                                            send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), response.to_string()));
                                        } else {
                                            // Send auth failure and close connection
                                            let response = serde_json::json!({
                                                "type": "auth_error",
                                                "error": "Invalid API key"
                                            });
                                            send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), response.to_string()));
                                            send_ws_push(channel_id, WsMessageType::Close, LazyLoadBlob::default());
                                        }
                                    }
                                }
                                "chat" => {
                                    // Process chat message
                                    if let Some(client) = self.chat_clients.get(&channel_id).cloned() {
                                        if let Ok(mut chat_request) = serde_json::from_value::<ChatRequest>(json_msg.get("payload").cloned().unwrap_or_default()) {
                                            // Set the API key from the authenticated client
                                            chat_request.api_key = client.api_key;

                                            // Process the chat request asynchronously
                                            match self.process_chat_request_with_streaming(chat_request, channel_id).await {
                                                Ok(response) => {
                                                    // Send final response
                                                    let ws_response = serde_json::json!({
                                                        "type": "chat_complete",
                                                        "payload": response
                                                    });
                                                    send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), ws_response.to_string()));
                                                }
                                                Err(e) => {
                                                    let error_response = serde_json::json!({
                                                        "type": "error",
                                                        "error": e
                                                    });
                                                    send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), error_response.to_string()));
                                                }
                                            }
                                        }
                                    } else {
                                        // Not authenticated
                                        let response = serde_json::json!({
                                            "type": "error",
                                            "error": "Not authenticated. Please send auth message first."
                                        });
                                        send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), response.to_string()));
                                    }
                                }
                                "ping" => {
                                    // Respond to ping with pong
                                    let response = serde_json::json!({
                                        "type": "pong"
                                    });
                                    send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), response.to_string()));
                                }
                                _ => {
                                    println!("Unknown message type from chat client: {}", msg_type);
                                }
                            }
                        }
                    }
                }
                WsMessageType::Close => {
                    // Clean up client connection
                    self.chat_clients.remove(&channel_id);
                    println!("Chat client {} disconnected", channel_id);
                }
                WsMessageType::Ping | WsMessageType::Pong => {
                    // Handle ping/pong for keepalive
                }

            }
        }
    }

    #[ws_client]
    fn handle_ws_client(&mut self, channel_id: u32, message_type: WsMessageType, blob: LazyLoadBlob) {
        match message_type {
            WsMessageType::Text | WsMessageType::Binary => {
                println!("Got WS Text");
                // Handle incoming message from the WebSocket server
                let message_bytes = blob.bytes;

                // Parse the message as JSON
                let message_str = String::from_utf8(message_bytes).unwrap_or_default();
                if let Ok(json_msg) = serde_json::from_str::<Value>(&message_str) {
                    self.handle_mcp_message(channel_id, json_msg);
                } else {
                    println!("Spider: Failed to parse MCP message from channel {}: {}", channel_id, message_str);
                }
            },
            WsMessageType::Close => {
                // Handle connection close
                println!("Spider: WebSocket connection closed for channel {}", channel_id);

                // Find and disconnect the server
                if let Some(conn) = self.ws_connections.remove(&channel_id) {
                    // Mark server as disconnected
                    if let Some(server) = self.mcp_servers.iter_mut().find(|s| s.id == conn.server_id) {
                        server.connected = false;
                        println!("Spider: MCP server {} disconnected", server.name);
                    }
                }

                // Clean up any pending requests for this connection
                self.pending_mcp_requests.retain(|_, req| {
                    if let Some(conn) = self.ws_connections.get(&channel_id) {
                        req.server_id != conn.server_id
                    } else {
                        true
                    }
                });
            },
            WsMessageType::Ping | WsMessageType::Pong => {
                // Ignore ping/pong messages for now
            }
        }
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
    async fn list_api_keys(&self) -> Result<Vec<ApiKeyInfo>, String> {
        let keys: Vec<ApiKeyInfo> = self.api_keys.iter().map(|(provider, key)| {
            ApiKeyInfo {
                provider: provider.clone(),
                created_at: key.created_at,
                last_used: key.last_used,
                key_preview: self.preview_key(&key.key),
            }
        }).collect();

        Ok(keys)
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
    async fn create_spider_key(&mut self, request: CreateSpiderKeyRequest) -> Result<SpiderApiKey, String> {
        let key = format!("sp_{}", Uuid::new_v4().to_string().replace("-", ""));

        let spider_key = SpiderApiKey {
            key: key.clone(),
            name: request.name,
            permissions: request.permissions,
            created_at: Utc::now().timestamp() as u64,
        };

        self.spider_api_keys.push(spider_key.clone());

        Ok(spider_key)
    }

    #[http]
    async fn list_spider_keys(&self) -> Result<Vec<SpiderApiKey>, String> {
        Ok(self.spider_api_keys.clone())
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
    async fn list_mcp_servers(&self) -> Result<Vec<McpServer>, String> {
        Ok(self.mcp_servers.clone())
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

        // For WebSocket-wrapped stdio servers, connect via WebSocket
        if transport.transport_type == "websocket" || transport.transport_type == "stdio" {
            // Get WebSocket URL (ws-mcp wrapper should be running)
            let ws_url = transport.url.clone()
                .unwrap_or_else(|| "ws://localhost:10125".to_string());

            // Allocate a channel ID for this connection
            let channel_id = self.next_channel_id;
            self.next_channel_id += 1;

            // Open WebSocket connection
            open_ws_connection(ws_url.clone(), None, channel_id).await
                .map_err(|e| format!("Failed to connect to MCP server: {:?}", e))?;

            // Store connection info
            self.ws_connections.insert(channel_id, WsConnection {
                server_id: server_id.clone(),
                server_name: server_name.clone(),
                channel_id,
                tools: Vec::new(),
                initialized: false,
            });

            // Send initialize request
            let init_request = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "clientInfo": {
                        "name": "spider",
                        "version": "1.0.0"
                    },
                    "capabilities": {}
                },
                "id": format!("init_{}", channel_id)
            });

            // Store pending request
            self.pending_mcp_requests.insert(
                format!("init_{}", channel_id),
                PendingMcpRequest {
                    request_id: format!("init_{}", channel_id),
                    conversation_id: None,
                    server_id: server_id.clone(),
                    request_type: McpRequestType::Initialize,
                }
            );

            // Send the initialize message
            let blob = LazyLoadBlob::new(Some("application/json"), init_request.to_string().into_bytes());
            send_ws_client_push(channel_id, WsMessageType::Text, blob);

            // Mark server as connecting (will be marked connected when initialized)
            if let Some(server) = self.mcp_servers.iter_mut().find(|s| s.id == server_id) {
                server.connected = false; // Will be set to true when initialization completes
            }

            Ok(format!("Connecting to MCP server {} via WebSocket...", server_name))
        } else {
            // For other transport types, use the old method for now
            let tools = self.discover_mcp_tools(&transport).await?;
            let tool_count = tools.len();

            // Update the server with discovered tools
            if let Some(server) = self.mcp_servers.iter_mut().find(|s| s.id == server_id) {
                server.tools = tools;
                server.connected = true;
            }

            Ok(format!("Connected to MCP server {} with {} tools", server_name, tool_count))
        }
    }

    #[http]
    async fn list_conversations(&self, request: ListConversationsRequest) -> Result<Vec<Conversation>, String> {
        let conversations: Vec<Conversation> = self.active_conversations.iter()
            .filter(|(_, conv)| {
                request.client.as_ref().map_or(true, |c| &conv.metadata.client == c)
            })
            .map(|(_, conv)| conv.clone())
            .skip(request.offset.unwrap_or(0) as usize)
            .take(request.limit.unwrap_or(50) as usize)
            .collect();

        Ok(conversations)
    }

    #[http]
    async fn get_conversation(&self, conversation_id: String) -> Result<Conversation, String> {
        // First check in-memory conversations
        for (id, conv) in &self.active_conversations {
            if id == &conversation_id {
                return Ok(conv.clone());
            }
        }

        // Try to load from VFS
        self.load_conversation_from_vfs(&conversation_id).await
    }

    #[http]
    async fn get_config(&self) -> Result<ConfigResponse, String> {
        Ok(ConfigResponse {
            default_llm_provider: self.default_llm_provider.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        })
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
    async fn chat(&mut self, request: ChatRequest) -> Result<ChatResponse, String> {
        // Use the shared internal chat processing logic (without WebSocket streaming)
        self.process_chat_internal(request, None).await
    }

    #[local]
    async fn process_request(&mut self, request: ProcessRequest) -> Result<ProcessResponse, String> {
        match request.action.as_str() {
            "chat" => {
                let chat_request: ChatRequest = serde_json::from_str(&request.payload)
                    .map_err(|e| format!("Invalid chat request: {}", e))?;
                let result = self.chat(chat_request).await?;
                let serialized = serde_json::to_string(&result)
                    .map_err(|e| format!("Failed to serialize chat response: {}", e))?;
                Ok(ProcessResponse {
                    success: true,
                    data: serialized,
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
        // For actual encryption, use base64 encoding with a marker
        // In production, this should use proper encryption with a key derivation function
        use base64::{Engine as _, engine::general_purpose};
        format!("encrypted:{}", general_purpose::STANDARD.encode(key.as_bytes()))
    }

    fn decrypt_key(&self, encrypted_key: &str) -> String {
        use base64::{Engine as _, engine::general_purpose};
        if encrypted_key.starts_with("encrypted:") {
            let encoded = &encrypted_key[10..];
            String::from_utf8(general_purpose::STANDARD.decode(encoded).unwrap_or_default())
                .unwrap_or_default()
        } else {
            encrypted_key.to_string()
        }
    }

    fn preview_key(&self, encrypted_key: &str) -> String {
        if encrypted_key.len() > 20 {
            format!("{}...", &encrypted_key[..20])
        } else {
            "***".to_string()
        }
    }

    fn validate_spider_key(&self, key: &str) -> bool {
        self.spider_api_keys.iter().any(|k| k.key == key)
    }

    // Streaming version of chat for WebSocket clients
    async fn process_chat_request_with_streaming(&mut self, request: ChatRequest, channel_id: u32) -> Result<ChatResponse, String> {
        // Send initial status
        let status_msg = serde_json::json!({
            "type": "status",
            "status": "processing",
            "message": "Starting chat processing..."
        });
        send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), status_msg.to_string()));

        // Use the regular chat processing but send streaming updates
        let result = self.process_chat_internal(request, Some(channel_id)).await;

        // Send completion status
        let status_msg = serde_json::json!({
            "type": "status",
            "status": "complete"
        });
        send_ws_push(channel_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), status_msg.to_string()));

        result
    }

    // Internal chat processing logic shared by HTTP and WebSocket
    async fn process_chat_internal(&mut self, request: ChatRequest, channel_id: Option<u32>) -> Result<ChatResponse, String> {
        // This is a refactored version of the chat logic that can send WebSocket updates
        // For now, just call the regular chat method
        // TODO: Refactor the chat method to use this shared logic

        // We can't easily call the #[http] method from here, so we'll need to duplicate the logic
        // or restructure the code. For now, let's just process it inline.

        // Validate Spider API key
        if !self.validate_spider_key(&request.api_key) {
            return Err("Unauthorized: Invalid Spider API key".to_string());
        }

        // Check permissions
        let spider_key = self.spider_api_keys.iter()
            .find(|k| k.key == request.api_key)
            .ok_or("Unauthorized: Invalid Spider API key")?;

        if !spider_key.permissions.contains(&"chat".to_string()) {
            return Err("Forbidden: API key lacks chat permission".to_string());
        }

        let conversation_id = Uuid::new_v4().to_string();
        let llm_provider = request.llm_provider.unwrap_or(self.default_llm_provider.clone());

        println!("Spider: Starting new conversation {} with provider {} (key: {})",
                 conversation_id, llm_provider, spider_key.name);

        // Get the API key for the selected provider and decrypt it
        let encrypted_key = self.api_keys.iter()
            .find(|(p, _)| p == &llm_provider)
            .map(|(_, k)| k.key.clone())
            .ok_or_else(|| format!("No API key found for provider: {}", llm_provider))?;
        let api_key = self.decrypt_key(&encrypted_key);

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

            // Send streaming update if WebSocket channel provided
            if let Some(ch_id) = channel_id {
                let stream_msg = serde_json::json!({
                    "type": "stream",
                    "iteration": iteration_count,
                    "message": format!("Processing iteration {}...", iteration_count)
                });
                send_ws_push(ch_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), stream_msg.to_string()));
            }

            // Call the LLM with available tools using the provider abstraction
            let provider = create_llm_provider(&llm_provider, &api_key);
            let llm_response = match provider.complete(&working_messages, &available_tools, self.max_tokens, self.temperature).await {
                Ok(response) => response,
                Err(e) => {
                    // Log the error for debugging
                    println!("Spider: Error calling LLM provider {}: {}", llm_provider, e);

                    // Check if it's an API key error
                    if e.contains("401") || e.contains("unauthorized") || e.contains("api key") {
                        return Err(format!("Authentication failed for {}: Please check your API key", llm_provider));
                    }

                    // Check if it's a rate limit error
                    if e.contains("429") || e.contains("rate limit") {
                        return Err(format!("Rate limited by {}: Please try again later", llm_provider));
                    }

                    // Return user-friendly error message
                    return Err(format!("Failed to get response from {}: {}", llm_provider, e));
                }
            };

            // Check if the response contains tool calls
            if let Some(ref tool_calls_json) = llm_response.tool_calls_json {
                // The agent wants to use tools - execute them
                println!("Spider: Iteration {} - Agent requested tool calls", iteration_count);

                // Send streaming update for tool calls
                if let Some(ch_id) = channel_id {
                    let stream_msg = serde_json::json!({
                        "type": "stream",
                        "iteration": iteration_count,
                        "message": "Executing tool calls...",
                        "tool_calls": tool_calls_json
                    });
                    send_ws_push(ch_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), stream_msg.to_string()));
                }

                let tool_results = self.process_tool_calls(tool_calls_json, Some(conversation_id.clone())).await?;

                // Add the assistant's message with tool calls
                working_messages.push(llm_response.clone());
                
                // Send the assistant message with tool calls to the client
                if let Some(ch_id) = channel_id {
                    let msg_update = serde_json::json!({
                        "type": "message",
                        "message": llm_response.clone()
                    });
                    send_ws_push(ch_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), msg_update.to_string()));
                }

                // Add tool results as a new message for the LLM to see
                let tool_message = Message {
                    role: "tool".to_string(),
                    content: "Tool execution results".to_string(),
                    tool_calls_json: None,
                    tool_results_json: Some(serde_json::to_string(&tool_results).unwrap()),
                    timestamp: Utc::now().timestamp() as u64,
                };
                working_messages.push(tool_message.clone());
                
                // Send the tool results message to the client
                if let Some(ch_id) = channel_id {
                    let msg_update = serde_json::json!({
                        "type": "message",
                        "message": tool_message
                    });
                    send_ws_push(ch_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), msg_update.to_string()));
                }

                // Continue the loop - the agent will decide what to do next
                continue;
            } else {
                // No tool calls - the agent has decided to provide a final response
                // Break the loop and return this response
                println!("Spider: Iteration {} - Agent provided final response (no tool calls)", iteration_count);

                // Send the final assistant message to the client
                if let Some(ch_id) = channel_id {
                    let msg_update = serde_json::json!({
                        "type": "message",
                        "message": llm_response.clone()
                    });
                    send_ws_push(ch_id, WsMessageType::Text, LazyLoadBlob::new(Some("application/json"), msg_update.to_string()));
                }

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

        // Get only the new messages that were added during this chat session
        // (everything after the initial user messages)
        let initial_message_count = request.messages.len();
        let new_messages = working_messages[initial_message_count..].to_vec();
        
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

        Ok(ChatResponse {
            conversation_id,
            response,
            all_messages: new_messages,
        })
    }

    async fn save_conversation_to_vfs(&self, conversation: &Conversation) -> Result<(), String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let filename = format!("{}-{}.json", timestamp, conversation.id);

        // Try to create the conversations directory if it doesn't exist
        let dir_path = match create_drive(our().package_id(), "conversations", None) {
            Ok(drive_path) => drive_path,
            Err(e) => {
                println!("Warning: Failed to create conversations drive: {:?}", e);
                // Continue anyway - we'll keep conversations in memory
                return Ok(());
            }
        };
        let file_path = format!("{dir_path}/{filename}");

        // Serialize the conversation
        let json_content = serde_json::to_string_pretty(conversation)
            .map_err(|e| format!("Failed to serialize conversation: {}", e))?;

        // Write to file
        match open_file(&file_path, true, None) {
            Ok(file) => {
                file.write(json_content.as_bytes())
                    .map_err(|e| format!("Failed to write conversation: {:?}", e))?;
                println!("Conversation {} saved to VFS at {}", conversation.id, file_path);
            }
            Err(e) => {
                println!("Warning: Failed to save conversation to VFS: {:?}", e);
                // Continue - conversation is still in memory
            }
        }

        Ok(())
    }

    async fn load_conversation_from_vfs(&self, conversation_id: &str) -> Result<Conversation, String> {
        let dir_path = format!("{}/conversations", our().node);

        // Open the conversations directory
        let dir = open_dir(&dir_path, false, None)
            .map_err(|e| format!("Failed to open conversations directory: {:?}", e))?;

        // List all files in the directory
        let entries = dir.read()
            .map_err(|e| format!("Failed to read directory: {:?}", e))?;

        // Look for a file containing the conversation ID
        for entry in entries {
            if entry.path.contains(conversation_id) {
                let file_path = format!("{}/{}", dir_path, entry.path);
                let file = open_file(&file_path, false, None)
                    .map_err(|e| format!("Failed to open conversation file: {:?}", e))?;

                let content = file.read()
                    .map_err(|e| format!("Failed to read conversation file: {:?}", e))?;

                let conversation: Conversation = serde_json::from_slice(&content)
                    .map_err(|e| format!("Failed to parse conversation: {}", e))?;

                return Ok(conversation);
            }
        }

        Err(format!("Conversation {} not found in VFS", conversation_id))
    }

    fn handle_mcp_message(&mut self, channel_id: u32, message: Value) {
        // Find the connection for this channel
        let conn = match self.ws_connections.get(&channel_id) {
            Some(c) => c.clone(),
            None => {
                println!("Spider: Received MCP message for unknown channel {}", channel_id);
                return;
            }
        };

        // Check if this is a response to a pending request
        if let Some(id) = message.get("id").and_then(|v| v.as_str()) {
            if let Some(pending) = self.pending_mcp_requests.remove(id) {
                match pending.request_type {
                    McpRequestType::Initialize => {
                        self.handle_initialize_response(channel_id, &conn, &message);
                    }
                    McpRequestType::ToolsList => {
                        self.handle_tools_list_response(channel_id, &conn, &message);
                    }
                    McpRequestType::ToolCall { tool_name: _ } => {
                        self.handle_tool_call_response(&pending, &message);
                    }
                }
            }
        }

        // Handle notifications or other messages
        if let Some(method) = message.get("method").and_then(|v| v.as_str()) {
            match method {
                "tools/list_changed" => {
                    // Tools have changed, re-fetch them
                    self.request_tools_list(channel_id);
                }
                _ => {
                    println!("Spider: Received MCP notification: {}", method);
                }
            }
        }
    }

    fn handle_initialize_response(&mut self, channel_id: u32, conn: &WsConnection, message: &Value) {
        if let Some(result) = message.get("result") {
            println!("Spider: MCP server {} initialized successfully", conn.server_name);

            // Mark connection as initialized
            if let Some(ws_conn) = self.ws_connections.get_mut(&channel_id) {
                ws_conn.initialized = true;
            }

            // Send notifications/initialized
            let notif = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            });
            let blob = LazyLoadBlob::new(Some("application/json"), notif.to_string().into_bytes());
            send_ws_client_push(channel_id, WsMessageType::Text, blob);

            // Request tools list
            self.request_tools_list(channel_id);
        } else if let Some(error) = message.get("error") {
            println!("Spider: Failed to initialize MCP server {}: {:?}", conn.server_name, error);
        }
    }

    fn request_tools_list(&mut self, channel_id: u32) {
        let request_id = format!("tools_{}", channel_id);
        let tools_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": request_id.clone()
        });

        // Store pending request
        if let Some(conn) = self.ws_connections.get(&channel_id) {
            self.pending_mcp_requests.insert(
                request_id.clone(),
                PendingMcpRequest {
                    request_id,
                    conversation_id: None,
                    server_id: conn.server_id.clone(),
                    request_type: McpRequestType::ToolsList,
                }
            );
        }

        let blob = LazyLoadBlob::new(Some("application/json"), tools_request.to_string().into_bytes());
        send_ws_client_push(channel_id, WsMessageType::Text, blob);
    }

    fn handle_tools_list_response(&mut self, channel_id: u32, conn: &WsConnection, message: &Value) {
        if let Some(result) = message.get("result") {
            if let Some(tools_json) = result.get("tools").and_then(|v| v.as_array()) {
                let mut tools = Vec::new();

                for tool_json in tools_json {
                    if let (Some(name), Some(description)) = (
                        tool_json.get("name").and_then(|v| v.as_str()),
                        tool_json.get("description").and_then(|v| v.as_str())
                    ) {
                        let parameters = tool_json.get("parameters")
                            .map(|p| p.to_string())
                            .unwrap_or_else(|| "{}".to_string());

                        tools.push(Tool {
                            name: name.to_string(),
                            description: description.to_string(),
                            parameters,
                        });
                    }
                }

                let tool_count = tools.len();
                println!("Spider: Received {} tools from MCP server {}", tool_count, conn.server_name);

                // Update connection with tools
                if let Some(ws_conn) = self.ws_connections.get_mut(&channel_id) {
                    ws_conn.tools = tools.clone();
                }

                // Update server with tools and mark as connected
                if let Some(server) = self.mcp_servers.iter_mut().find(|s| s.id == conn.server_id) {
                    server.tools = tools;
                    server.connected = true;
                }
            }
        } else if let Some(error) = message.get("error") {
            println!("Spider: Failed to get tools from MCP server {}: {:?}", conn.server_name, error);
        }
    }

    fn handle_tool_call_response(&mut self, pending: &PendingMcpRequest, message: &Value) {
        // TODO: Route tool call results back to the appropriate conversation
        println!("Spider: Received tool call response for conversation {:?}: {:?}",
                 pending.conversation_id, message);

        // This will be implemented when we handle tool calls in chat
    }

    async fn discover_mcp_tools(&self, transport: &TransportConfig) -> Result<Vec<Tool>, String> {
        // MCP tool discovery implementation
        match transport.transport_type.as_str() {
            "stdio" => {
                // In WASM environment, we can't spawn processes
                // Return example tools for demonstration
                // In production, this would use a proxy service or HTTP transport
                println!("Note: stdio transport not fully supported in WASM environment");
                println!("Returning example tools for MCP server");

                Ok(vec![
                    Tool {
                        name: "search".to_string(),
                        description: "Search for information".to_string(),
                        parameters: r#"{"type":"object","properties":{"query":{"type":"string","description":"The search query"}},"required":["query"]}"#.to_string(),
                    },
                    Tool {
                        name: "calculate".to_string(),
                        description: "Perform mathematical calculations".to_string(),
                        parameters: r#"{"type":"object","properties":{"expression":{"type":"string","description":"The mathematical expression to evaluate"}},"required":["expression"]}"#.to_string(),
                    },
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

    async fn execute_mcp_tool(&mut self, server_id: &str, tool_name: &str, parameters: &Value, conversation_id: Option<String>) -> Result<Value, String> {
        let server = self.mcp_servers.iter()
            .find(|s| s.id == server_id && s.connected)
            .ok_or_else(|| format!("MCP server {} not found or not connected", server_id))?;

        // Check if the tool exists
        let _tool = server.tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| format!("Tool {} not found on server {}", tool_name, server_id))?;

        // Find the WebSocket connection for this server
        let channel_id = self.ws_connections.iter()
            .find(|(_, conn)| conn.server_id == server_id)
            .map(|(id, _)| *id)
            .ok_or_else(|| format!("No WebSocket connection found for server {}", server_id))?;

        // Execute the tool based on transport type
        match server.transport.transport_type.as_str() {
            "stdio" | "websocket" => {
                // Execute via WebSocket
                let request_id = format!("tool_{}_{}", channel_id, Uuid::new_v4());
                let tool_request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "tools/call",
                    "params": {
                        "name": tool_name,
                        "arguments": parameters
                    },
                    "id": request_id.clone()
                });

                // Store pending request
                self.pending_mcp_requests.insert(
                    request_id.clone(),
                    PendingMcpRequest {
                        request_id: request_id.clone(),
                        conversation_id,
                        server_id: server_id.to_string(),
                        request_type: McpRequestType::ToolCall { tool_name: tool_name.to_string() },
                    }
                );

                // Send the tool call
                let blob = LazyLoadBlob::new(Some("application/json"), tool_request.to_string().into_bytes());
                send_ws_client_push(channel_id, WsMessageType::Text, blob);

                // For now, return a placeholder response
                // In a production system, we would wait for the response asynchronously
                // and route it back to the appropriate conversation
                Ok(serde_json::json!({
                    "status": "executing",
                    "request_id": request_id,
                    "tool": tool_name,
                    "server": server_id
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

    async fn process_tool_calls(&mut self, tool_calls_json: &str, conversation_id: Option<String>) -> Result<Vec<ToolResult>, String> {
        let tool_calls: Vec<ToolCall> = serde_json::from_str(tool_calls_json)
            .map_err(|e| format!("Failed to parse tool calls: {}", e))?;

        let mut results = Vec::new();

        for tool_call in tool_calls {
            // Find which MCP server has this tool and get its ID
            let server_id = self.mcp_servers.iter()
                .find(|s| s.connected && s.tools.iter().any(|t| t.name == tool_call.tool_name))
                .map(|s| s.id.clone());

            let result = if let Some(server_id) = server_id {
                let params: Value = serde_json::from_str(&tool_call.parameters)
                    .unwrap_or(Value::Object(serde_json::Map::new()));

                match self.execute_mcp_tool(&server_id, &tool_call.tool_name, &params, conversation_id.clone()).await {
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

// LLM Provider Abstraction
trait LlmProvider {
    fn complete<'a>(&'a self, messages: &'a [Message], tools: &'a [Tool], max_tokens: u32, temperature: f32)
        -> Pin<Box<dyn Future<Output = Result<Message, String>> + 'a>>;
    fn name(&self) -> &str;
}

struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for AnthropicProvider {
    fn complete<'a>(&'a self, messages: &'a [Message], tools: &'a [Tool], max_tokens: u32, temperature: f32)
        -> Pin<Box<dyn Future<Output = Result<Message, String>> + 'a>> {
        Box::pin(async move {
            // For simplicity in WASM, skip retry logic for now
            self.complete_with_retry(messages, tools, max_tokens, temperature).await
        })
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}

impl AnthropicProvider {
    async fn complete_with_retry(&self, messages: &[Message], tools: &[Tool], max_tokens: u32, temperature: f32) -> Result<Message, String> {
        // Initialize the Anthropic SDK client
        let client = AnthropicClient::new(&self.api_key);

        // Convert our Message format to SDK Message format
        let mut sdk_messages = Vec::new();

        for msg in messages {
            let role = match msg.role.as_str() {
                "user" => Role::User,
                "assistant" => Role::Assistant,
                "tool" => Role::User, // Tool results are sent as user messages in Claude API
                _ => Role::User,
            };

            // Handle different message types
            let content = if let Some(tool_results_json) = &msg.tool_results_json {
                // Parse tool results and format them for the SDK
                let tool_results: Vec<ToolResult> = serde_json::from_str(tool_results_json)
                    .unwrap_or_else(|_| Vec::new());

                // Format tool results as text content
                let mut result_text = String::from("Tool execution results:\n");
                for result in tool_results {
                    result_text.push_str(&format!("- Tool call {}: {}\n", result.tool_call_id, result.result));
                }
                Content::Text(result_text)
            } else if let Some(_tool_calls_json) = &msg.tool_calls_json {
                // For now, include tool calls as text in the message
                // The SDK will handle tool use blocks separately
                Content::Text(format!("{}\n[Tool calls pending]", msg.content))
            } else {
                Content::Text(msg.content.clone())
            };

            sdk_messages.push(SdkMessage { role, content });
        }

        // Convert our Tool format to SDK Tool format
        let sdk_tools: Vec<SdkTool> = tools.iter().map(|tool| {
            let params = serde_json::from_str::<Value>(&tool.parameters)
                .unwrap_or_else(|_| serde_json::json!({}));

            // Extract required fields from the schema
            let required = params.get("required")
                .and_then(|r| r.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            SdkTool::new(
                tool.name.clone(),
                tool.description.clone(),
                params.clone(),
                required,
            )
        }).collect();

        // Create the request
        let mut request = CreateMessageRequest::new(
            //"claude-opus-4-1-20250805",
            "claude-sonnet-4-20250514",
            sdk_messages,
            max_tokens,
        ).with_temperature(temperature);

        // Add tools if any
        if !sdk_tools.is_empty() {
            request = request.with_tools(sdk_tools)
                .with_tool_choice(ToolChoice::Auto {
                    disable_parallel_tool_use: Some(false)
                });
        }

        // Send the message using the SDK
        let response = client.send_message(request).await
            .map_err(|e| format!("Failed to send message via SDK: {:?}", e))?;

        // Convert SDK response back to our Message format
        let mut content_text = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();

        for block in &response.content {
            match block {
                ResponseContentBlock::Text { text, .. } => {
                    if !content_text.is_empty() {
                        content_text.push(' ');
                    }
                    content_text.push_str(text);
                }
                ResponseContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id: id.clone(),
                        tool_name: name.clone(),
                        parameters: serde_json::to_string(input)
                            .unwrap_or_else(|_| "{}".to_string()),
                    });
                }
            }
        }

        Ok(Message {
            role: "assistant".to_string(),
            content: content_text,
            tool_calls_json: if tool_calls.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&tool_calls).unwrap())
            },
            tool_results_json: None,
            timestamp: Utc::now().timestamp() as u64,
        })
    }
}

// Placeholder for future providers
struct OpenAIProvider {
    api_key: String,
}

impl OpenAIProvider {
    fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl LlmProvider for OpenAIProvider {
    fn complete<'a>(&'a self, _messages: &'a [Message], _tools: &'a [Tool], _max_tokens: u32, _temperature: f32)
        -> Pin<Box<dyn Future<Output = Result<Message, String>> + 'a>> {
        Box::pin(async move {
            Err("OpenAI provider not yet implemented".to_string())
        })
    }

    fn name(&self) -> &str {
        "openai"
    }
}

fn create_llm_provider(provider_type: &str, api_key: &str) -> Box<dyn LlmProvider> {
    match provider_type {
        "anthropic" => Box::new(AnthropicProvider::new(api_key.to_string())),
        "openai" => Box::new(OpenAIProvider::new(api_key.to_string())),
        _ => Box::new(AnthropicProvider::new(api_key.to_string())), // Default to Anthropic
    }
}
