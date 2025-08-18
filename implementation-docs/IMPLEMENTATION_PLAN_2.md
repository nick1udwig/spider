# Spider MCP Client - Implementation Plan 2
## Remaining Features and Improvements

## Assessment of Current Implementation

The engineer has made significant progress on the Spider MCP client. Here's what has been completed:

### ✅ Completed Features
1. **Application renamed** from skeleton-app to spider
2. **Core state structure** implemented with all required types
3. **API key management** - set, list, remove, encrypt
4. **Spider API key management** - create, list, revoke
5. **MCP server management** - add, list, connect, tool discovery (placeholder)
6. **Conversation management** - list, get, basic structure
7. **Configuration management** - get and update config
8. **Basic agentic loop** - placeholder implementation with simulated tool calls
9. **Frontend structure** - all UI components created with navigation
10. **Zustand store** - complete state management setup
11. **P2P local endpoint** - process_request handler implemented

### ⚠️ Partially Implemented Features
1. **Anthropic API integration** - Structure exists but using placeholder responses
2. **MCP tool execution** - Discovery and execution are placeholders
3. **VFS storage** - Methods exist but not actually saving/loading from VFS
4. **Chat functionality** - Frontend exists but sendMessage not implemented

### ❌ Missing Features
1. **Actual Anthropic SDK integration** - Not imported or used
2. **Real MCP server communication** - No stdio/HTTP transport implementation
3. **VFS file operations** - No actual file I/O to spider:ware.hypr/conversations/
4. **HTTP authentication** - Spider API keys not validated in HTTP requests
5. **Frontend chat implementation** - sendMessage function is empty
6. **Generated TypeScript bindings** - Not using target/ui/caller-utils.ts

## Detailed Implementation Plan for Remaining Features

### Phase 1: Critical Integration Fixes

#### 1.1 Integrate Anthropic SDK
**File:** `spider/Cargo.toml`
```toml
[dependencies.hyperware_anthropic_sdk]
git = "https://github.com/hyperware-ai/hyperware-anthropic-sdk"
```

**File:** `spider/src/lib.rs`
- Import the SDK: `use hyperware_anthropic_sdk::{Client, Message as AnthropicMessage, ToolDefinition};`
- Replace the placeholder `call_anthropic_api` method with actual SDK usage:
```rust
async fn call_anthropic_api(&self, api_key: &str, messages: &[Message], tools: &[Tool]) -> Result<Message, String> {
    let client = Client::new(api_key);
    let anthropic_messages = convert_to_anthropic_format(messages);
    let tool_definitions = convert_tools_to_anthropic(tools);
    
    let response = client
        .messages()
        .model("claude-3-opus-20240229")
        .max_tokens(self.max_tokens)
        .temperature(self.temperature)
        .messages(anthropic_messages)
        .tools(tool_definitions)
        .send()
        .await
        .map_err(|e| format!("Anthropic API error: {}", e))?;
    
    convert_anthropic_response_to_message(response)
}
```

#### 1.2 Implement VFS Storage
**File:** `spider/src/lib.rs`

Add VFS imports:
```rust
use hyperware_process_lib::{
    vfs::{VfsDirEntry, VfsAction, VfsResponse, open_file, create_file},
    get_blob,
};
```

Implement actual VFS operations:
```rust
async fn save_conversation_to_vfs(&self, conversation: &Conversation) -> Result<(), String> {
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("{}.jsonl", timestamp);
    let path = format!("spider:ware.hypr/conversations/{}", filename);
    
    let json_line = serde_json::to_string(conversation)
        .map_err(|e| format!("Failed to serialize conversation: {}", e))?;
    
    // Create directory if it doesn't exist
    let create_dir_request = Request::to(("our", "vfs", "distro", "sys"))
        .body(serde_json::to_vec(&VfsAction::CreateDir {
            path: "spider:ware.hypr/conversations".to_string(),
            package_id: our().process.package_id.clone(),
        }).unwrap());
    
    let _ = send::<VfsResponse>(create_dir_request).await;
    
    // Write the conversation file
    let create_file_request = Request::to(("our", "vfs", "distro", "sys"))
        .body(serde_json::to_vec(&VfsAction::CreateFile {
            path,
            package_id: our().process.package_id.clone(),
        }).unwrap())
        .blob(LazyLoadBlob {
            mime: Some("application/jsonl".to_string()),
            bytes: json_line.into_bytes(),
        });
    
    send::<VfsResponse>(create_file_request).await
        .map_err(|e| format!("Failed to save conversation: {:?}", e))?;
    
    Ok(())
}

async fn load_conversation_from_vfs(&self, conversation_id: &str) -> Result<Conversation, String> {
    // List all conversation files
    let list_request = Request::to(("our", "vfs", "distro", "sys"))
        .body(serde_json::to_vec(&VfsAction::ReadDir {
            path: "spider:ware.hypr/conversations".to_string(),
            package_id: our().process.package_id.clone(),
        }).unwrap());
    
    let response = send::<VfsResponse>(list_request).await
        .map_err(|e| format!("Failed to list conversations: {:?}", e))?;
    
    // Search through files for the conversation
    // Implementation details...
}
```

### Phase 2: MCP Server Communication

#### 2.1 Implement Stdio Transport
**File:** `spider/src/mcp_transport.rs` (new file)
```rust
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead, Write};
use tokio::sync::mpsc;

pub struct StdioTransport {
    command: String,
    args: Vec<String>,
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
}

impl StdioTransport {
    pub async fn connect(&mut self) -> Result<(), String> {
        let mut child = Command::new(&self.command)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP process: {}", e))?;
        
        // Set up bidirectional communication
        // Read stdout in a separate task
        // Handle JSON-RPC messages
    }
    
    pub async fn send_request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        // Send JSON-RPC request
        // Wait for response
    }
}
```

#### 2.2 Implement HTTP Transport
```rust
pub struct HttpTransport {
    base_url: String,
    client: reqwest::Client,
}

impl HttpTransport {
    pub async fn send_request(&self, method: &str, params: Value) -> Result<Value, String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": uuid::Uuid::new_v4().to_string(),
        });
        
        let response = self.client
            .post(&self.base_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        
        // Parse JSON-RPC response
    }
}
```

#### 2.3 Update MCP Tool Discovery and Execution
```rust
async fn discover_mcp_tools(&self, transport: &TransportConfig) -> Result<Vec<Tool>, String> {
    let transport = create_transport(transport)?;
    
    // Send tools/list request
    let response = transport.send_request("tools/list", json!({})).await?;
    
    // Parse tool definitions
    let tools = parse_mcp_tools(response)?;
    Ok(tools)
}

async fn execute_mcp_tool(&self, server_id: &str, tool_name: &str, parameters: &Value) -> Result<Value, String> {
    let server = self.get_server(server_id)?;
    let transport = create_transport(&server.transport)?;
    
    // Send tool call request
    let response = transport.send_request("tools/call", json!({
        "name": tool_name,
        "arguments": parameters,
    })).await?;
    
    Ok(response)
}
```

### Phase 3: Frontend Chat Implementation

#### 3.1 Complete Chat Functionality
**File:** `ui/src/store/spider.ts`
```typescript
sendMessage: async (message: string) => {
  try {
    set({ isLoading: true, error: null });
    
    // Get current conversation or create new one
    let conversation = get().activeConversation;
    if (!conversation) {
      conversation = {
        id: '',
        messages: [],
        metadata: {
          start_time: new Date().toISOString(),
          client: 'web-ui',
          from_stt: false,
        },
        llm_provider: get().config.default_llm_provider,
        mcp_servers: get().mcpServers.filter(s => s.connected).map(s => s.id),
      };
    }
    
    // Add user message
    const userMessage: Message = {
      role: 'user',
      content: message,
      timestamp: Date.now(),
    };
    
    conversation.messages.push(userMessage);
    set({ activeConversation: { ...conversation } });
    
    // Send to backend
    const response = await api.chat({
      messages: conversation.messages,
      llm_provider: conversation.llm_provider,
      mcp_servers: conversation.mcp_servers,
      metadata: conversation.metadata,
    });
    
    // Update conversation with response
    conversation.id = response.conversation_id;
    conversation.messages.push(response.response);
    
    set({ 
      activeConversation: { ...conversation },
      isLoading: false 
    });
  } catch (error: any) {
    set({ 
      error: error.message || 'Failed to send message', 
      isLoading: false 
    });
  }
},
```

#### 3.2 Use Generated TypeScript Bindings
**File:** `ui/src/utils/api.ts`

Replace manual API calls with generated bindings:
```typescript
import * as caller from '../../target/ui/caller-utils';

export const setApiKey = (provider: string, key: string) => 
  caller.setApiKey({ provider, key });

export const listApiKeys = () => 
  caller.listApiKeys();

export const chat = (request: ChatRequest) => 
  caller.chat(request);

// etc. for all API methods
```

### Phase 4: Authentication and Security

#### 4.1 Implement HTTP Authentication Middleware
**File:** `spider/src/lib.rs`

Add authentication check for external HTTP requests:
```rust
#[http]
async fn chat(&mut self, request: ChatRequest) -> Result<String, String> {
    // Extract Spider API key from request or headers
    let api_key = extract_api_key(&request)?;
    
    if !self.validate_spider_key(&api_key) {
        return Err("Unauthorized: Invalid Spider API key".to_string());
    }
    
    // Check permissions
    let spider_key = self.spider_api_keys.iter()
        .find(|k| k.key == api_key)
        .ok_or("Invalid API key")?;
    
    if !spider_key.permissions.contains(&"chat".to_string()) {
        return Err("Forbidden: API key lacks chat permission".to_string());
    }
    
    // Continue with chat logic...
}
```

### Phase 5: Additional LLM Providers

#### 5.1 Abstract LLM Interface
**File:** `spider/src/llm_providers/mod.rs` (new file)
```rust
#[async_trait]
pub trait LlmProvider {
    async fn complete(&self, messages: Vec<Message>, tools: Vec<Tool>) -> Result<Message, String>;
    fn name(&self) -> &str;
}

pub struct AnthropicProvider {
    client: hyperware_anthropic_sdk::Client,
}

pub struct OpenAIProvider {
    api_key: String,
}

pub struct GeminiProvider {
    api_key: String,
}

pub fn create_provider(provider_type: &str, api_key: &str) -> Box<dyn LlmProvider> {
    match provider_type {
        "anthropic" => Box::new(AnthropicProvider::new(api_key)),
        "openai" => Box::new(OpenAIProvider::new(api_key)),
        "google" => Box::new(GeminiProvider::new(api_key)),
        _ => panic!("Unknown provider"),
    }
}
```

### Phase 6: Testing and Polish

#### 6.1 Add Error Recovery
- Implement retry logic for API calls
- Add circuit breaker for MCP servers
- Graceful degradation when services unavailable

#### 6.2 Add Rate Limiting
```rust
struct RateLimiter {
    requests_per_minute: HashMap<String, Vec<u64>>,
    max_rpm: u32,
}

impl RateLimiter {
    fn check_and_update(&mut self, api_key: &str) -> Result<(), String> {
        // Check if rate limit exceeded
        // Update request count
    }
}
```

#### 6.3 Add Metrics and Logging
```rust
struct UsageMetrics {
    total_requests: u64,
    tokens_used: HashMap<String, u64>,
    tool_calls: HashMap<String, u64>,
}
```

## Implementation Priority

1. **Critical (Week 1)**
   - Anthropic SDK integration
   - VFS storage implementation
   - Complete chat functionality in frontend
   - Use generated TypeScript bindings

2. **Important (Week 2)**
   - Real MCP server communication (stdio transport)
   - HTTP authentication for Spider API keys
   - Error handling and recovery

3. **Nice to Have (Week 3+)**
   - Additional LLM providers (OpenAI, Gemini)
   - HTTP transport for MCP
   - Rate limiting
   - Metrics and analytics

## Testing Requirements

1. **Unit Tests**
   - API key encryption/decryption
   - Message format conversions
   - Tool call parsing

2. **Integration Tests**
   - Anthropic API communication
   - MCP server connection
   - VFS file operations

3. **End-to-End Tests**
   - Complete conversation flow
   - Tool execution loop
   - Multi-turn conversations

## Notes for Implementation

1. **Anthropic SDK**: Check the exact import path and API from the hyperware-anthropic-sdk repository
2. **VFS Operations**: Ensure proper PackageId is used for all VFS operations
3. **MCP Protocol**: Follow the Model Context Protocol specification for JSON-RPC communication
4. **Generated Bindings**: Run `kit build --hyperapp` to generate TypeScript bindings after backend changes
5. **Error Handling**: Always provide meaningful error messages for debugging
6. **Security**: Never log full API keys, always validate permissions

## Success Criteria

The Spider MCP client will be considered complete when:
1. ✅ Can connect to Claude API and get real responses
2. ✅ Can connect to at least one MCP server via stdio
3. ✅ Can execute tool calls in an agentic loop
4. ✅ Saves all conversations to VFS
5. ✅ Frontend can send messages and display responses
6. ✅ Spider API keys provide authenticated access
7. ✅ All core endpoints are functional