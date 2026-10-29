import { useEffect, useRef } from 'react';
import { Download, Trash2 } from 'lucide-react';
import { useLogStore, LogCategory } from '../stores/logStore';

export default function LogTerminal() {
  const { entries, filter, setFilter, clearLogs } = useLogStore();
  const bottomRef = useRef<HTMLDivElement>(null);

  const filteredEntries = filter === 'ALL' 
    ? entries 
    : entries.filter(e => e.category === filter);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [filteredEntries]);

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'success': return 'text-emerald-400';
      case 'info': return 'text-blue-400';
      case 'warn': return 'text-amber-400';
      case 'error': return 'text-red-400';
      case 'debug': return 'text-gray-500';
      default: return 'text-gray-300';
    }
  };

  const handleExport = () => {
    const text = entries.map(e => `[${e.timestamp.toISOString()}] [${e.category}] [${e.level}] ${e.message}`).join('\n');
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `voidrelay_logs_${new Date().toISOString()}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const categories: LogCategory[] = ['ALL', 'BT', 'STT', 'LLM', 'TTS', 'CALLS', 'SYSTEM'];

  return (
    <div className="flex flex-col h-full bg-[#0d0d12] rounded-xl border border-white/10 overflow-hidden font-mono text-xs shadow-2xl">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-black/40 border-b border-white/5">
        <div className="flex gap-2 overflow-x-auto no-scrollbar">
          {categories.map(cat => (
            <button
              key={cat}
              onClick={() => setFilter(cat)}
              className={`px-2 py-1 rounded text-xs transition-colors ${filter === cat ? 'bg-primary/20 text-primary' : 'text-gray-500 hover:text-gray-300'}`}
            >
              {cat}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <button onClick={handleExport} className="p-1.5 text-gray-500 hover:text-white transition-colors" title="Export Logs">
            <Download className="w-4 h-4" />
          </button>
          <button onClick={clearLogs} className="p-1.5 text-gray-500 hover:text-danger transition-colors" title="Clear Logs">
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Log Output */}
      <div className="flex-1 overflow-y-auto p-4 space-y-1">
        {filteredEntries.map(entry => (
          <div key={entry.id} className="flex gap-3 hover:bg-white/[0.02] px-1 rounded">
            <span className="text-gray-600 shrink-0">{entry.timestamp.toLocaleTimeString([], { hour12: false })}</span>
            <span className={`w-12 shrink-0 text-[10px] uppercase font-bold opacity-70 ${getLevelColor(entry.level)}`}>{entry.category}</span>
            <span className={`break-words ${getLevelColor(entry.level)}`}>{entry.message}</span>
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
