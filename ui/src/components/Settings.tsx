import { useState, useEffect } from 'react';
import { useSpiderStore } from '../store/spider';

export default function Settings() {
  const { config, isLoading, error, updateConfig } = useSpiderStore();
  const [provider, setProvider] = useState(config.default_llm_provider);
  const [maxTokens, setMaxTokens] = useState(config.max_tokens);
  const [temperature, setTemperature] = useState(config.temperature);

  useEffect(() => {
    setProvider(config.default_llm_provider);
    setMaxTokens(config.max_tokens);
    setTemperature(config.temperature);
  }, [config]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await updateConfig({
      default_llm_provider: provider,
      max_tokens: maxTokens,
      temperature: temperature,
    });
  };

  return (
    <div className="component-container">
      <div className="component-header">
        <h2>Settings</h2>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="settings-form">
        <div className="form-group">
          <label htmlFor="provider">Default LLM Provider</label>
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
          <label htmlFor="max-tokens">Max Tokens</label>
          <input
            id="max-tokens"
            type="number"
            value={maxTokens}
            onChange={(e) => setMaxTokens(Number(e.target.value))}
            min="1"
            max="100000"
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="temperature">Temperature</label>
          <input
            id="temperature"
            type="number"
            value={temperature}
            onChange={(e) => setTemperature(Number(e.target.value))}
            min="0"
            max="2"
            step="0.1"
          />
        </div>
        
        <button type="submit" className="btn btn-primary" disabled={isLoading}>
          {isLoading ? 'Saving...' : 'Save Settings'}
        </button>
      </form>
    </div>
  );
}