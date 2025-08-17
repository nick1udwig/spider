import { useState } from 'react';
import { useSpiderStore } from '../store/spider';

export default function McpServers() {
  const { mcpServers, isLoading, error, addMcpServer, connectMcpServer } = useSpiderStore();
  const [showAddForm, setShowAddForm] = useState(false);
  const [serverName, setServerName] = useState('');
  const [transportType, setTransportType] = useState<'stdio' | 'http'>('stdio');
  const [command, setCommand] = useState('');
  const [args, setArgs] = useState('');
  const [url, setUrl] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    const transport = transportType === 'stdio' 
      ? { Stdio: { command, args: args.split(' ').filter(a => a) } }
      : { Http: { url } };
    
    await addMcpServer(serverName, transport);
    setServerName('');
    setCommand('');
    setArgs('');
    setUrl('');
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
            <select
              id="transport-type"
              value={transportType}
              onChange={(e) => setTransportType(e.target.value as 'stdio' | 'http')}
            >
              <option value="stdio">Stdio</option>
              <option value="http">HTTP</option>
            </select>
          </div>
          
          {transportType === 'stdio' ? (
            <>
              <div className="form-group">
                <label htmlFor="command">Command</label>
                <input
                  id="command"
                  type="text"
                  value={command}
                  onChange={(e) => setCommand(e.target.value)}
                  placeholder="/path/to/mcp-server"
                  required
                />
              </div>
              
              <div className="form-group">
                <label htmlFor="args">Arguments (space-separated)</label>
                <input
                  id="args"
                  type="text"
                  value={args}
                  onChange={(e) => setArgs(e.target.value)}
                  placeholder="--arg1 value1 --arg2 value2"
                />
              </div>
            </>
          ) : (
            <div className="form-group">
              <label htmlFor="url">URL</label>
              <input
                id="url"
                type="url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://mcp-server.example.com"
                required
              />
            </div>
          )}
          
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
                  Transport: {server.transport.Stdio 
                    ? `Stdio: ${server.transport.Stdio.command}` 
                    : `HTTP: ${server.transport.Http?.url}`}
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