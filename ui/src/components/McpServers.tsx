import { useState, useEffect } from 'react';
import { useSpiderStore } from '../store/spider';

export default function McpServers() {
  const { 
    mcpServers, 
    isLoading, 
    error, 
    addMcpServer, 
    connectMcpServer, 
    disconnectMcpServer,
    removeMcpServer,
    loadMcpServers 
  } = useSpiderStore();
  const [showAddForm, setShowAddForm] = useState(false);
  const [serverName, setServerName] = useState('');
  const [url, setUrl] = useState('ws://localhost:10125');
  const [connectingServers, setConnectingServers] = useState<Set<string>>(new Set());

  // Periodically refresh server status
  useEffect(() => {
    const interval = setInterval(() => {
      loadMcpServers();
    }, 5000); // Refresh every 5 seconds

    return () => clearInterval(interval);
  }, [loadMcpServers]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const transport = {
      transportType: 'websocket',
      command: null,
      args: null,
      url: url
    };
    
    await addMcpServer(serverName, transport);
    setServerName('');
    setUrl('ws://localhost:10125');
    setShowAddForm(false);
    
    // Refresh servers list after adding
    setTimeout(() => loadMcpServers(), 500);
  };

  const handleConnect = async (serverId: string) => {
    setConnectingServers(prev => new Set(prev).add(serverId));
    try {
      await connectMcpServer(serverId);
      // Poll for connection status update
      let attempts = 0;
      const pollInterval = setInterval(async () => {
        await loadMcpServers();
        attempts++;
        const server = mcpServers.find(s => s.id === serverId);
        if (server?.connected || attempts > 10) {
          clearInterval(pollInterval);
          setConnectingServers(prev => {
            const next = new Set(prev);
            next.delete(serverId);
            return next;
          });
        }
      }, 500);
    } catch (error) {
      setConnectingServers(prev => {
        const next = new Set(prev);
        next.delete(serverId);
        return next;
      });
    }
  };

  const handleDisconnect = async (serverId: string) => {
    await disconnectMcpServer(serverId);
    // Refresh servers list after disconnecting
    setTimeout(() => loadMcpServers(), 500);
  };

  const handleRemove = async (serverId: string) => {
    if (confirm('Are you sure you want to remove this MCP server?')) {
      await removeMcpServer(serverId);
      // Refresh servers list after removing
      setTimeout(() => loadMcpServers(), 500);
    }
  };

  return (
    <div className="component-container">
      <div className="component-header">
        <h2>MCP Servers</h2>
        <button 
          className="btn btn-primary"
          onClick={() => setShowAddForm(!showAddForm)}
        >
          {showAddForm ? 'Cancel' : 'Add MCP Server'}
        </button>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      {showAddForm && (
        <form onSubmit={handleSubmit} className="mcp-server-form">
          <div className="form-group">
            <label htmlFor="server-name">Server Name</label>
            <input
              id="server-name"
              type="text"
              value={serverName}
              onChange={(e) => setServerName(e.target.value)}
              placeholder="My MCP Server"
              required
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="transport-type">Transport Type</label>
            <div className="transport-info">WebSocket</div>
          </div>
          
          <div className="form-group">
            <label htmlFor="url">WebSocket URL</label>
            <input
              id="url"
              type="text"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="ws://localhost:10125"
              required
            />
            <small className="form-help">
              URL of the WebSocket MCP server or ws-mcp wrapper
            </small>
          </div>
          
          <button type="submit" className="btn btn-primary" disabled={isLoading}>
            {isLoading ? 'Adding...' : 'Add Server'}
          </button>
        </form>
      )}

      <div className="component-content">
        <div className="mcp-servers-list">
          {mcpServers.length === 0 ? (
            <p className="empty-state">No MCP servers configured</p>
          ) : (
          mcpServers.map((server) => {
            const isConnecting = connectingServers.has(server.id);
            return (
              <div key={server.id} className="mcp-server-item">
                <div className="mcp-server-info">
                  <h3>{server.name}</h3>
                  <p>
                    Status: {
                      isConnecting ? '🟡 Connecting...' :
                      server.connected ? '🟢 Connected' : 
                      '🔴 Disconnected'
                    }
                  </p>
                  <p>
                    Transport: WebSocket - {server.transport.url || 'No URL specified'}
                  </p>
                  <p>Tools: {server.tools.length}</p>
                  {server.tools.length > 0 && (
                    <details className="mcp-server-tools">
                      <summary>Available Tools</summary>
                      <ul>
                        {server.tools.map((tool, index) => (
                          <li key={index}>
                            <strong>{tool.name}</strong>: {tool.description}
                          </li>
                        ))}
                      </ul>
                    </details>
                  )}
                </div>
                <div className="mcp-server-actions">
                  {!server.connected && !isConnecting && (
                    <button
                      className="btn btn-success"
                      onClick={() => handleConnect(server.id)}
                      disabled={isLoading}
                    >
                      Connect
                    </button>
                  )}
                  {server.connected && (
                    <button
                      className="btn btn-warning"
                      onClick={() => handleDisconnect(server.id)}
                      disabled={isLoading}
                    >
                      Disconnect
                    </button>
                  )}
                  <button
                    className="btn btn-danger btn-icon"
                    onClick={() => handleRemove(server.id)}
                    disabled={isLoading}
                    title="Remove server"
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <line x1="18" y1="6" x2="6" y2="18"></line>
                      <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                  </button>
                </div>
              </div>
            );
          })
          )}
        </div>
      </div>
    </div>
  );
}