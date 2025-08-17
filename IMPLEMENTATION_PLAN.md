# Spider MCP Client Implementation Plan

## Overview
Spider is an MCP (Model Context Protocol) client and conversation repository for the Hyperware platform. It provides agentic tool use capabilities with LLMs (starting with Anthropic's Claude API) and stores all conversations with metadata for future retrieval.

## Core Architecture

### 1. Backend Structure (Rust)

#### 1.1 Rename Application
- Rename from `skeleton-app` to `spider` throughout the codebase
- Update `metadata.json`:
  - name: "Spider"
  - package_name: "spider"
  - publisher: "spider.os"
  - description: "MCP client and conversation repository"
- Update `Cargo.toml` workspace and package names
- Update folder name from `skeleton-app/` to `spider/`
- Update wit_world to "spider-dot-os-v0"

#### 1.2 Core State Structure
```rust
pub struct SpiderState {
    // API Configuration
    api_keys: HashMap<String, ApiKey>,  // Store as Vec<(String, ApiKey)> for WIT
    spider_api_keys: Vec<SpiderApiKey>,
    
    // MCP Server Configuration
    mcp_servers: Vec<McpServer>,
    
    // Conversation Management
    active_conversations: HashMap<String, Conversation>, // Store as Vec<(String, Conversation)>
    
    // Configuration
    default_llm_provider: String,
    max_tokens: u32,
    temperature: f32,
}

struct ApiKey {
    provider: String,  // "anthropic", "openai", "google"
    key: String,       // encrypted
    created_at: u64,
    last_used: Option<u64>,
}

struct SpiderApiKey {
    key: String,
    name: String,
    permissions: Vec<String>,
    created_at: u64,
}

struct McpServer {
    id: String,
    name: String,
    transport: TransportConfig,
    tools: Vec<Tool>,
    connected: bool,
}

struct Conversation {
    id: String,
    messages: Vec<Message>,
    metadata: ConversationMetadata,
    llm_provider: String,
    mcp_servers: Vec<String>,
}

struct ConversationMetadata {
    start_time: String,
    client: String,
    from_stt: bool,
}

struct Message {
    role: String,  // "user", "assistant", "system", "tool"
    content: String,
    tool_calls: Option<Vec<ToolCall>>,
    tool_results: Option<Vec<ToolResult>>,
    timestamp: u64,
}
```

#### 1.3 Dependencies to Add
Add to `spider/Cargo.toml`:
```toml
# HTTP client for API calls
reqwest = { version = "0.11", features = ["json"] }
# Async runtime utilities
tokio = { version = "1.35", features = ["sync"] }
# UUID generation
uuid = { version = "1.4.1", features = ["v4", "serde"] }
# Time handling
chrono = { version = "0.4", features = ["serde"] }
# Encryption for API keys
sha2 = "0.10"
base64 = "0.21"
# Message Pack for p2p communication
rmp-serde = "1.1"
```

Also add the Anthropic SDK:
```toml
[dependencies.hyperware_anthropic_sdk]
git = "https://github.com/hyperware-ai/hyperware-anthropic-sdk"
```

### 2. API Endpoints

#### 2.1 HTTP Endpoints for Frontend

```rust
// API Key Management
#[http]
async fn set_api_key(&mut self, request: SetApiKeyRequest) -> Result<String, String>

#[http]
async fn list_api_keys(&self, _request_body: String) -> String  // Returns JSON

#[http]
async fn remove_api_key(&mut self, provider: String) -> Result<String, String>

// Spider API Key Management
#[http]
async fn create_spider_key(&mut self, request: CreateSpiderKeyRequest) -> Result<SpiderApiKey, String>

#[http]
async fn list_spider_keys(&self, _request_body: String) -> String  // Returns JSON

#[http]
async fn revoke_spider_key(&mut self, key_id: String) -> Result<String, String>

// MCP Server Management
#[http]
async fn add_mcp_server(&mut self, request: AddMcpServerRequest) -> Result<String, String>

#[http]
async fn list_mcp_servers(&self, _request_body: String) -> String  // Returns JSON

#[http]
async fn connect_mcp_server(&mut self, server_id: String) -> Result<String, String>

// Conversation History
#[http]
async fn list_conversations(&self, request: ListConversationsRequest) -> String  // Returns JSON

#[http]
async fn get_conversation(&self, conversation_id: String) -> String  // Returns JSON

// Configuration
#[http]
async fn get_config(&self, _request_body: String) -> String  // Returns JSON

#[http]
async fn update_config(&mut self, request: UpdateConfigRequest) -> Result<String, String>
```

#### 2.2 HTTP Endpoints for External Clients

```rust
// Main chat endpoint for external clients (requires Spider API key)
#[http]
async fn chat(&mut self, request: ChatRequest) -> Result<ChatResponse, String>
```

#### 2.3 Local P2P Endpoints

```rust
// Local endpoint for other Hyperware processes
#[local]
async fn process_request(&mut self, request: ProcessRequest) -> Result<ProcessResponse, String>
```

### 3. Core Functionality Implementation

#### 3.1 LLM Integration
- Start with Anthropic API using the hyperware-anthropic-sdk
- Design with abstraction layer for future providers:
  ```rust
  trait LlmProvider {
      async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, String>;
      async fn stream_complete(&self, messages: Vec<Message>) -> Result<Stream, String>;
  }
  ```

#### 3.2 MCP Server Integration
- Connect to MCP servers via stdio or HTTP transport
- Discover available tools from servers
- Route tool calls to appropriate servers
- Handle tool responses

#### 3.3 Agentic Tool Loop
1. Receive user request
2. Send to LLM with available tools
3. If LLM calls tools:
   - Execute tool calls against MCP servers
   - Send results back to LLM
   - Repeat until LLM provides final response
4. Return final response to user

#### 3.4 Conversation Storage
- Store conversations in VFS at `spider:ware.hypr/conversations/`
- Use JSONL format with filename: `YYYYMMDD-HHMMSS.jsonl`
- Each line is a complete conversation object
- Implement indexing for fast retrieval

### 4. Frontend Implementation (TypeScript/React)

#### 4.1 Pages/Views

1. **Dashboard** (`/`)
   - Quick stats (conversations today, API usage)
   - Recent conversations list
   - Quick chat interface

2. **API Keys** (`/api-keys`)
   - Add/remove LLM API keys
   - Test connection status
   - Usage statistics

3. **Spider Keys** (`/spider-keys`)
   - Generate new Spider API keys
   - List existing keys with permissions
   - Revoke keys

4. **MCP Servers** (`/mcp-servers`)
   - Add new MCP server connections
   - View connected servers and available tools
   - Test tool execution

5. **Conversations** (`/conversations`)
   - Browse conversation history
   - Search by metadata
   - View full conversation threads

6. **Settings** (`/settings`)
   - Default LLM provider
   - Model parameters (temperature, max tokens)
   - Export/import configuration

#### 4.2 State Management (Zustand)
```typescript
interface SpiderStore {
  // API Keys
  apiKeys: ApiKey[];
  spiderKeys: SpiderApiKey[];
  
  // MCP Servers
  mcpServers: McpServer[];
  
  // Conversations
  conversations: Conversation[];
  activeConversation: Conversation | null;
  
  // Configuration
  config: SpiderConfig;
  
  // Actions
  setApiKey: (key: ApiKey) => Promise<void>;
  removeApiKey: (provider: string) => Promise<void>;
  createSpiderKey: (request: CreateSpiderKeyRequest) => Promise<SpiderApiKey>;
  addMcpServer: (server: McpServer) => Promise<void>;
  sendMessage: (message: string) => Promise<void>;
  loadConversations: () => Promise<void>;
}
```

#### 4.3 UI Components
- ApiKeyManager: Form for adding/managing API keys
- McpServerCard: Display MCP server status and tools
- ConversationView: Render conversation with proper formatting
- ToolCallDisplay: Show tool invocations and results
- ChatInterface: Main chat input and response display

### 5. Security Considerations

1. **API Key Storage**
   - Encrypt API keys before storing
   - Never log or display full keys
   - Implement key rotation reminders

2. **Spider API Keys**
   - Generate cryptographically secure keys
   - Implement permission system
   - Rate limiting per key

3. **HTTP Authentication**
   - Require Spider API key in Authorization header
   - Validate permissions for each endpoint

4. **P2P Security**
   - Verify source process identity
   - Only accept local requests for sensitive operations

### 6. VFS Structure

```
spider:ware.hypr/
├── conversations/
│   ├── 20250816-141005.jsonl
│   ├── 20250816-152030.jsonl
│   └── ...
├── config/
│   └── settings.json
└── keys/
    └── encrypted_keys.json
```

### 7. Build and Deployment

1. Update build scripts for new name
2. Generate WIT bindings with `kit build --hyperapp`
3. Test locally with `kit start-packages`
4. Ensure manifest.json includes required capabilities:
   - vfs:distro:sys (for file storage)
   - http-client:distro:sys (for API calls)

### 8. Testing Strategy

1. Unit tests for core logic
2. Integration tests for MCP server communication
3. Mock LLM responses for testing tool loops
4. Frontend component tests
5. End-to-end conversation flow tests

### 9. Implementation Order

1. **Phase 1: Core Backend**
   - Rename application
   - Set up basic state structure
   - Implement API key management endpoints
   - Add VFS integration for storage

2. **Phase 2: LLM Integration**
   - Integrate Anthropic SDK
   - Implement basic chat endpoint
   - Add conversation storage

3. **Phase 3: MCP Support**
   - Add MCP server connection logic
   - Implement tool discovery
   - Build agentic tool loop

4. **Phase 4: Frontend**
   - Build API key management UI
   - Create chat interface
   - Add conversation history view

5. **Phase 5: P2P Support**
   - Implement local endpoints
   - Add process authentication
   - Test with other Hyperware apps

6. **Phase 6: Polish**
   - Add error handling
   - Implement rate limiting
   - Add usage analytics
   - Create documentation

## Notes for Implementor

- Refer to `resources/example-apps/sign/` for local messaging patterns
- Refer to `resources/example-apps/file-explorer/` for VFS usage patterns
- Use `resources/guides/04-P2P-PATTERNS.md` for p2p implementation details
- Remember that WIT types have limitations - use JSON strings for complex data
- Always test with `kit build --hyperapp` to generate proper bindings
- The frontend must include `/our.js` script before any other scripts
- Use the generated `target/ui/caller-utils.ts` for all backend API calls