import { useState, useEffect } from 'react';
import './App.css';
import { useSpiderStore } from './store/spider';
import ApiKeys from './components/ApiKeys';
import SpiderKeys from './components/SpiderKeys';
import McpServers from './components/McpServers';
import Conversations from './components/Conversations';
import Chat from './components/Chat';
import Settings from './components/Settings';

type TabType = 'chat' | 'api-keys' | 'spider-keys' | 'mcp-servers' | 'conversations' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<TabType>('chat');
  const { initialize } = useSpiderStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  return (
    <div className="app">
      <header className="app-header">
        <div className="app-title">
          <span className="app-icon">🕷️</span>
          <h1>Spider MCP Client</h1>
        </div>
        <nav className="app-nav">
          <button 
            className={`nav-btn ${activeTab === 'chat' ? 'active' : ''}`}
            onClick={() => setActiveTab('chat')}
          >
            Chat
          </button>
          <button 
            className={`nav-btn ${activeTab === 'api-keys' ? 'active' : ''}`}
            onClick={() => setActiveTab('api-keys')}
          >
            API Keys
          </button>
          <button 
            className={`nav-btn ${activeTab === 'spider-keys' ? 'active' : ''}`}
            onClick={() => setActiveTab('spider-keys')}
          >
            Spider Keys
          </button>
          <button 
            className={`nav-btn ${activeTab === 'mcp-servers' ? 'active' : ''}`}
            onClick={() => setActiveTab('mcp-servers')}
          >
            MCP Servers
          </button>
          <button 
            className={`nav-btn ${activeTab === 'conversations' ? 'active' : ''}`}
            onClick={() => setActiveTab('conversations')}
          >
            History
          </button>
          <button 
            className={`nav-btn ${activeTab === 'settings' ? 'active' : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            Settings
          </button>
        </nav>
      </header>
      
      <main className="app-main">
        {activeTab === 'chat' && <Chat />}
        {activeTab === 'api-keys' && <ApiKeys />}
        {activeTab === 'spider-keys' && <SpiderKeys />}
        {activeTab === 'mcp-servers' && <McpServers />}
        {activeTab === 'conversations' && <Conversations />}
        {activeTab === 'settings' && <Settings />}
      </main>
    </div>
  );
}

export default App;