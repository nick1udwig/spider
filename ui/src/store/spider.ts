import { create } from 'zustand';
import * as api from '../utils/api';

interface ApiKey {
  provider: string;
  created_at: number;
  last_used?: number;
  key_preview: string;
}

interface SpiderApiKey {
  key: string;
  name: string;
  permissions: string[];
  created_at: number;
}

interface McpServer {
  id: string;
  name: string;
  transport: {
    Stdio?: { command: string; args: string[] };
    Http?: { url: string };
  };
  tools: Array<{
    name: string;
    description: string;
    parameters: string;
  }>;
  connected: boolean;
}

interface Conversation {
  id: string;
  messages: Message[];
  metadata: {
    start_time: string;
    client: string;
    from_stt: boolean;
  };
  llm_provider: string;
  mcp_servers: string[];
}

interface Message {
  role: string;
  content: string;
  tool_calls?: Array<{
    id: string;
    tool_name: string;
    parameters: string;
  }>;
  tool_results?: Array<{
    tool_call_id: string;
    result: string;
  }>;
  timestamp: number;
}

interface SpiderConfig {
  default_llm_provider: string;
  max_tokens: number;
  temperature: number;
}

interface SpiderStore {
  // State
  apiKeys: ApiKey[];
  spiderKeys: SpiderApiKey[];
  mcpServers: McpServer[];
  conversations: Conversation[];
  activeConversation: Conversation | null;
  config: SpiderConfig;
  isLoading: boolean;
  error: string | null;
  isConnected: boolean;
  nodeId: string;
  
  // Actions
  initialize: () => Promise<void>;
  setApiKey: (provider: string, key: string) => Promise<void>;
  removeApiKey: (provider: string) => Promise<void>;
  loadApiKeys: () => Promise<void>;
  createSpiderKey: (name: string, permissions: string[]) => Promise<void>;
  revokeSpiderKey: (key: string) => Promise<void>;
  loadSpiderKeys: () => Promise<void>;
  addMcpServer: (name: string, transport: any) => Promise<void>;
  connectMcpServer: (serverId: string) => Promise<void>;
  loadMcpServers: () => Promise<void>;
  sendMessage: (message: string) => Promise<void>;
  loadConversations: (client?: string, limit?: number) => Promise<void>;
  loadConversation: (id: string) => Promise<void>;
  loadConfig: () => Promise<void>;
  updateConfig: (config: Partial<SpiderConfig>) => Promise<void>;
  clearError: () => void;
}

export const useSpiderStore = create<SpiderStore>((set, get) => ({
  // Initial state
  apiKeys: [],
  spiderKeys: [],
  mcpServers: [],
  conversations: [],
  activeConversation: null,
  config: {
    default_llm_provider: 'anthropic',
    max_tokens: 4096,
    temperature: 0.7,
  },
  isLoading: false,
  error: null,
  isConnected: false,
  nodeId: '',

  // Actions
  initialize: async () => {
    try {
      set({ isLoading: true });
      
      // Check if our.js is loaded
      if (typeof window.our === 'undefined') {
        set({ 
          isConnected: false, 
          error: 'Not connected to Hyperware node. Make sure you are running on a Hyperware node.',
          isLoading: false 
        });
        return;
      }

      const nodeId = window.our.node;
      set({ isConnected: true, nodeId });

      // Load initial data
      await Promise.all([
        get().loadApiKeys(),
        get().loadSpiderKeys(),
        get().loadMcpServers(),
        get().loadConfig(),
      ]);
      
      set({ isLoading: false });
    } catch (error: any) {
      set({ 
        error: error.message || 'Failed to initialize', 
        isLoading: false,
        isConnected: false 
      });
    }
  },

  setApiKey: async (provider: string, key: string) => {
    try {
      set({ isLoading: true, error: null });
      await api.setApiKey(provider, key);
      await get().loadApiKeys();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to set API key', isLoading: false });
    }
  },

  removeApiKey: async (provider: string) => {
    try {
      set({ isLoading: true, error: null });
      await api.removeApiKey(provider);
      await get().loadApiKeys();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to remove API key', isLoading: false });
    }
  },

  loadApiKeys: async () => {
    try {
      const keys = await api.listApiKeys();
      set({ apiKeys: keys });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load API keys' });
    }
  },

  createSpiderKey: async (name: string, permissions: string[]) => {
    try {
      set({ isLoading: true, error: null });
      await api.createSpiderKey(name, permissions);
      await get().loadSpiderKeys();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to create Spider key', isLoading: false });
    }
  },

  revokeSpiderKey: async (key: string) => {
    try {
      set({ isLoading: true, error: null });
      await api.revokeSpiderKey(key);
      await get().loadSpiderKeys();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to revoke Spider key', isLoading: false });
    }
  },

  loadSpiderKeys: async () => {
    try {
      const keys = await api.listSpiderKeys();
      set({ spiderKeys: keys });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load Spider keys' });
    }
  },

  addMcpServer: async (name: string, transport: any) => {
    try {
      set({ isLoading: true, error: null });
      await api.addMcpServer(name, transport);
      await get().loadMcpServers();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to add MCP server', isLoading: false });
    }
  },

  connectMcpServer: async (serverId: string) => {
    try {
      set({ isLoading: true, error: null });
      await api.connectMcpServer(serverId);
      await get().loadMcpServers();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to connect MCP server', isLoading: false });
    }
  },

  loadMcpServers: async () => {
    try {
      const servers = await api.listMcpServers();
      set({ mcpServers: servers });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load MCP servers' });
    }
  },

  sendMessage: async (message: string) => {
    // TODO: Implement chat functionality
    console.log('Sending message:', message);
  },

  loadConversations: async (client?: string, limit?: number) => {
    try {
      const conversations = await api.listConversations(client, limit);
      set({ conversations });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load conversations' });
    }
  },

  loadConversation: async (id: string) => {
    try {
      const conversation = await api.getConversation(id);
      set({ activeConversation: conversation });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load conversation' });
    }
  },

  loadConfig: async () => {
    try {
      const config = await api.getConfig();
      set({ config });
    } catch (error: any) {
      set({ error: error.message || 'Failed to load config' });
    }
  },

  updateConfig: async (config: Partial<SpiderConfig>) => {
    try {
      set({ isLoading: true, error: null });
      await api.updateConfig(config);
      await get().loadConfig();
      set({ isLoading: false });
    } catch (error: any) {
      set({ error: error.message || 'Failed to update config', isLoading: false });
    }
  },

  clearError: () => set({ error: null }),
}));