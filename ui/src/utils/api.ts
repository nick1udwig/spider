// @ts-nocheck
// Import the generated functions directly
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
  listConversations as _listConversations,
  getConversation as _getConversation,
  getConfig as _getConfig,
  updateConfig as _updateConfig,
  chat as _chat,
} from '@caller-utils';

export async function setApiKey(provider: string, key: string) {
  return _setApiKey({ provider, key });
}

export async function listApiKeys() {
  const response = await _listApiKeys();
  return JSON.parse(response);
}

export async function removeApiKey(provider: string) {
  return _removeApiKey(provider);
}

export async function createSpiderKey(name: string, permissions: string[]) {
  const response = await _createSpiderKey({ name, permissions });
  return JSON.parse(response);
}

export async function listSpiderKeys() {
  const response = await _listSpiderKeys();
  return JSON.parse(response);
}

export async function revokeSpiderKey(key: string) {
  return _revokeSpiderKey(key);
}

export async function addMcpServer(name: string, transport: any) {
  return _addMcpServer({ name, transport });
}

export async function listMcpServers() {
  const response = await _listMcpServers();
  return JSON.parse(response);
}

export async function connectMcpServer(serverId: string) {
  return _connectMcpServer(serverId);
}

export async function listConversations(client?: string, limit?: number, offset?: number) {
  const response = await _listConversations({
    client: client || null,
    limit: limit || null,
    offset: offset || null
  });
  return JSON.parse(response);
}

export async function getConversation(conversationId: string) {
  const response = await _getConversation(conversationId);
  return JSON.parse(response);
}

export async function getConfig() {
  const response = await _getConfig();
  return JSON.parse(response);
}

export async function updateConfig(config: any) {
  return _updateConfig({
    defaultLlmProvider: config.default_llm_provider || null,
    maxTokens: config.max_tokens || null,
    temperature: config.temperature || null
  });
}

export async function chat(apiKey: string, messages: any[], llmProvider?: string, mcpServers?: string[], metadata?: any) {
  const response = await _chat({
    apiKey,
    messages,
    llmProvider: llmProvider || null,
    mcpServers: mcpServers || null,
    metadata: metadata || null
  });
  return JSON.parse(response);
}