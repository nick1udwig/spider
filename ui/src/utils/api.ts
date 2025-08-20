// Import the generated functions and types directly
import {
  setApiKey as _setApiKey,
  listApiKeys as _listApiKeys,
  removeApiKey as _removeApiKey,
  createSpiderKey as _createSpiderKey,
  listSpiderKeys as _listSpiderKeys,
  revokeSpiderKey as _revokeSpiderKey,
  addMcpServer as _addMcpServer,
  listMcpServers as _listMcpServers,
  connectMcpServer as _connectMcpServer,
  disconnectMcpServer as _disconnectMcpServer,
  removeMcpServer as _removeMcpServer,
  listConversations as _listConversations,
  getConversation as _getConversation,
  getConfig as _getConfig,
  updateConfig as _updateConfig,
  chat as _chat,
  type ApiKeyInfo,
  type SpiderApiKey,
  type McpServer,
  type Conversation,
  type ConfigResponse,
  type ChatResponse,
  type Message,
  type ConversationMetadata,
  type TransportConfig,
} from '@caller-utils';

export async function setApiKey(provider: string, key: string) {
  return _setApiKey({ provider, key });
}

export async function listApiKeys(): Promise<ApiKeyInfo[]> {
  return _listApiKeys();
}

export async function removeApiKey(provider: string) {
  return _removeApiKey(provider);
}

export async function createSpiderKey(name: string, permissions: string[]): Promise<SpiderApiKey> {
  return _createSpiderKey({ name, permissions });
}

export async function listSpiderKeys(): Promise<SpiderApiKey[]> {
  return _listSpiderKeys();
}

export async function revokeSpiderKey(key: string) {
  return _revokeSpiderKey(key);
}

export async function addMcpServer(name: string, transport: TransportConfig): Promise<string> {
  return _addMcpServer({ name, transport });
}

export async function listMcpServers(): Promise<McpServer[]> {
  return _listMcpServers();
}

export async function connectMcpServer(serverId: string) {
  return _connectMcpServer(serverId);
}

export async function disconnectMcpServer(serverId: string) {
  return _disconnectMcpServer(serverId);
}

export async function removeMcpServer(serverId: string) {
  return _removeMcpServer(serverId);
}

export async function listConversations(client?: string, limit?: number, offset?: number): Promise<Conversation[]> {
  return _listConversations({
    client: client || null,
    limit: limit || null,
    offset: offset || null
  });
}

export async function getConversation(conversationId: string): Promise<Conversation> {
  return _getConversation(conversationId);
}

export async function getConfig(): Promise<ConfigResponse> {
  return _getConfig();
}

export async function updateConfig(config: Partial<ConfigResponse>): Promise<string> {
  return _updateConfig({
    defaultLlmProvider: config.defaultLlmProvider || null,
    maxTokens: config.maxTokens || null,
    temperature: config.temperature || null
  });
}

export async function chat(apiKey: string, messages: Message[], llmProvider?: string, mcpServers?: string[], metadata?: ConversationMetadata, signal?: AbortSignal): Promise<ChatResponse> {
  // TODO: Pass signal to the underlying API call when supported
  return _chat({
    apiKey,
    messages,
    llmProvider: llmProvider || null,
    mcpServers: mcpServers || null,
    metadata: metadata || null
  });
}