import { useEffect, useRef } from 'react';
import { User, Bot } from 'lucide-react';
import { useCallStore } from '../stores/callStore';

export default function ConversationView() {
  const { conversationHistory, llmStreamContent } = useCallStore();
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [conversationHistory, llmStreamContent]);

  if (conversationHistory.length === 0 && !llmStreamContent) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-500 bg-black/10 rounded-xl border border-white/5 p-6">
        <Bot className="w-12 h-12 mb-3 opacity-20" />
        <p>Waiting for conversation to begin...</p>
      </div>
    );
  }

  return (
    <div 
      ref={scrollRef}
      className="flex flex-col gap-4 overflow-y-auto h-full p-4 bg-black/20 rounded-xl border border-white/5 scroll-smooth"
    >
      {conversationHistory.map((msg) => (
        <div 
          key={msg.id} 
          className={`flex gap-3 max-w-[85%] ${msg.role === 'user' ? 'self-end flex-row-reverse' : 'self-start'}`}
        >
          <div className={`shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-1 ${msg.role === 'user' ? 'bg-primary/20 text-primary' : 'bg-secondary/20 text-secondary'}`}>
            {msg.role === 'user' ? <User className="w-4 h-4" /> : <Bot className="w-4 h-4" />}
          </div>
          <div className={`p-3 rounded-2xl ${msg.role === 'user' ? 'bg-primary/20 text-white rounded-tr-sm' : 'bg-surface border border-white/10 text-gray-200 rounded-tl-sm'}`}>
            <p className="text-sm leading-relaxed whitespace-pre-wrap">{msg.content}</p>
          </div>
        </div>
      ))}
      
      {/* Streaming Response Indicator */}
      {llmStreamContent && (
        <div className="flex gap-3 max-w-[85%] self-start animate-in fade-in">
          <div className="shrink-0 w-8 h-8 rounded-full bg-secondary/20 text-secondary flex items-center justify-center mt-1">
            <Bot className="w-4 h-4" />
          </div>
          <div className="p-3 rounded-2xl bg-surface border border-white/10 text-gray-200 rounded-tl-sm">
            <p className="text-sm leading-relaxed whitespace-pre-wrap">
              {llmStreamContent}
              <span className="inline-block w-1.5 h-4 ml-1 bg-secondary animate-pulse align-middle" />
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
