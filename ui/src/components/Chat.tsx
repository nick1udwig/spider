import { useState } from 'react';
import { useSpiderStore } from '../store/spider';

export default function Chat() {
  const { activeConversation, isLoading, sendMessage } = useSpiderStore();
  const [message, setMessage] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!message.trim()) return;
    
    await sendMessage(message);
    setMessage('');
  };

  return (
    <div className="chat-container">
      <div className="chat-messages">
        {activeConversation?.messages.map((msg, index) => (
          <div key={index} className={`message message-${msg.role}`}>
            <div className="message-role">{msg.role}</div>
            <div className="message-content">{msg.content}</div>
            {msg.tool_calls && (
              <div className="tool-calls">
                {msg.tool_calls.map(call => (
                  <div key={call.id} className="tool-call">
                    Tool: {call.tool_name}
                  </div>
                ))}
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