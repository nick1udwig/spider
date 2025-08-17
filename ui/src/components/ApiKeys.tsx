import { useState } from 'react';
import { useSpiderStore } from '../store/spider';

export default function ApiKeys() {
  const { apiKeys, isLoading, error, setApiKey, removeApiKey } = useSpiderStore();
  const [showAddForm, setShowAddForm] = useState(false);
  const [provider, setProvider] = useState('anthropic');
  const [apiKeyValue, setApiKeyValue] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!apiKeyValue.trim()) return;
    
    await setApiKey(provider, apiKeyValue);
    setApiKeyValue('');
    setShowAddForm(false);
  };

  return (
    <div className="component-container">
      <div className="component-header">
        <h2>API Keys</h2>
        <button 
          className="btn btn-primary"
          onClick={() => setShowAddForm(!showAddForm)}
        >
          {showAddForm ? 'Cancel' : 'Add API Key'}
        </button>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      {showAddForm && (
        <form onSubmit={handleSubmit} className="api-key-form">
          <div className="form-group">
            <label htmlFor="provider">Provider</label>
            <select
              id="provider"
              value={provider}
              onChange={(e) => setProvider(e.target.value)}
            >
              <option value="anthropic">Anthropic</option>
              <option value="openai">OpenAI</option>
              <option value="google">Google</option>
            </select>
          </div>
          
          <div className="form-group">
            <label htmlFor="api-key">API Key</label>
            <input
              id="api-key"
              type="password"
              value={apiKeyValue}
              onChange={(e) => setApiKeyValue(e.target.value)}
              placeholder="sk-..."
              required
            />
          </div>
          
          <button type="submit" className="btn btn-primary" disabled={isLoading}>
            {isLoading ? 'Adding...' : 'Add Key'}
          </button>
        </form>
      )}

      <div className="api-keys-list">
        {apiKeys.length === 0 ? (
          <p className="empty-state">No API keys configured</p>
        ) : (
          apiKeys.map((key) => (
            <div key={key.provider} className="api-key-item">
              <div className="api-key-info">
                <h3>{key.provider}</h3>
                <p>Key: {key.key_preview}</p>
                <p>Created: {new Date(key.created_at * 1000).toLocaleDateString()}</p>
                {key.last_used && (
                  <p>Last used: {new Date(key.last_used * 1000).toLocaleDateString()}</p>
                )}
              </div>
              <button
                className="btn btn-danger"
                onClick={() => removeApiKey(key.provider)}
                disabled={isLoading}
              >
                Remove
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}