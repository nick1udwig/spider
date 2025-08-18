import { useState } from 'react';
import { useSpiderStore } from '../store/spider';

export default function Chat() {
  const { activeConversation, isLoading, error, sendMessage } = useSpiderStore();
  const [message, setMessage] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;
    
    await sendMessage(message);
    setMessage('');
  };

  return (
    <div className="chat-container">
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}
      
      <div className="chat-messages">
        {activeConversation?.messages.map((msg, index) => (
          <div key={index} className={`message message-${msg.role}`}>
            <div className="message-role">{msg.role}</div>
            <div className="message-content">{msg.content}</div>
            {msg.toolCallsJson && (
              <div className="tool-calls">
                <pre>{msg.toolCallsJson}</pre>
              </div>
            )}
            {msg.toolResultsJson && (
              <div className="tool-results">
                <pre>{msg.toolResultsJson}</pre>
              </div>
            )}
          </div>
        )) || (
          <div className="empty-chat">
            <p>Start a conversation by typing a message below</p>
          </div>
        )}
      </div>
      
      <form onSubmit={handleSubmit} className="chat-input-form">
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder="Type your message..."
          disabled={isLoading}
          className="chat-input"
        />
        <button type="submit" disabled={isLoading || !message.trim()} className="btn btn-primary">
          {isLoading ? 'Sending...' : 'Send'}
        </button>
      </form>
    </div>
  );
}