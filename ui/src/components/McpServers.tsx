import { useState } from 'react';
import { useSpiderStore } from '../store/spider';

export default function McpServers() {
  const { mcpServers, isLoading, error, addMcpServer, connectMcpServer } = useSpiderStore();
  const [showAddForm, setShowAddForm] = useState(false);
  const [serverName, setServerName] = useState('');
  const [url, setUrl] = useState('ws://localhost:10125');

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

      <div className="mcp-servers-list">
        {mcpServers.length === 0 ? (
          <p className="empty-state">No MCP servers configured</p>
        ) : (
          mcpServers.map((server) => (
            <div key={server.id} className="mcp-server-item">
              <div className="mcp-server-info">
                <h3>{server.name}</h3>
                <p>Status: {server.connected ? '🟢 Connected' : '🔴 Disconnected'}</p>
                <p>
                  Transport: WebSocket - {server.transport.url || 'No URL specified'}
                </p>
                <p>Tools: {server.tools.length}</p>
              </div>
              <div className="mcp-server-actions">
                {!server.connected && (
                  <button
                    className="btn btn-success"
                    onClick={() => connectMcpServer(server.id)}
                    disabled={isLoading}
                  >
                    Connect
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}