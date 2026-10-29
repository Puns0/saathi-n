import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, Download, Trash2, ChevronDown, ChevronUp } from 'lucide-react';

interface CallRecord {
  id: string;
  timestamp: string;
  number: string;
  duration_sec: number;
  platform: string;
  language: string;
  summary: string;
  full_transcript: string;
}

export default function CallHistory() {
  const [history, setHistory] = useState<CallRecord[]>([]);
  const [search, setSearch] = useState('');
  const [expandedId, setExpandedId] = useState<string | null>(null);

  useEffect(() => {
    loadHistory();
  }, []);

  const loadHistory = async () => {
    try {
      const recs = await invoke<CallRecord[]>('get_call_history', { limit: 100 });
      setHistory(recs);
    } catch (e) {
      console.error(e);
    }
  };

  const handleExport = async () => {
    try {
      await invoke('export_call_history', { path: './voidrelay_history_export.json' });
      alert('Exported successfully');
    } catch(e) {
      console.error(e);
    }
  };

  const handleClear = async () => {
    if (confirm('Are you sure you want to clear all call history?')) {
      try {
        await invoke('clear_call_history');
        setHistory([]);
      } catch(e) {
        console.error(e);
      }
    }
  };

  const filtered = history.filter(h => 
    h.number.includes(search) || 
    h.summary.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
      <header className="flex justify-between items-end">
        <div>
          <h2 className="text-3xl font-bold tracking-tight mb-2">Call Archives</h2>
          <p className="text-gray-400">Review past conversations, summaries, and transcripts.</p>
        </div>
        <div className="flex gap-3">
          <button onClick={handleExport} className="btn-secondary flex items-center gap-2"><Download className="w-4 h-4" /> Export</button>
          <button onClick={handleClear} className="btn-secondary text-danger border-danger/30 hover:bg-danger/10 flex items-center gap-2"><Trash2 className="w-4 h-4" /> Clear</button>
        </div>
      </header>

      <div className="glass-panel p-6">
        <div className="flex items-center gap-3 mb-6 bg-black/40 border border-white/10 rounded-lg px-4 py-2">
          <Search className="w-5 h-5 text-gray-500" />
          <input 
            type="text" 
            placeholder="Search numbers or summaries..." 
            className="bg-transparent border-none outline-none text-white w-full"
            value={search}
            onChange={e => setSearch(e.target.value)}
          />
        </div>

        <div className="space-y-4">
          {filtered.length === 0 ? (
            <div className="text-center py-12 text-gray-500">No call history found.</div>
          ) : filtered.map(call => (
            <div key={call.id} className="border border-white/5 rounded-xl bg-black/20 overflow-hidden transition-colors hover:border-white/10">
              <div 
                className="p-4 flex items-center justify-between cursor-pointer"
                onClick={() => setExpandedId(expandedId === call.id ? null : call.id)}
              >
                <div className="flex items-center gap-6">
                  <div className="text-sm">
                    <div className="text-gray-400">{new Date(call.timestamp).toLocaleString()}</div>
                    <div className="font-bold text-lg">{call.number}</div>
                  </div>
                  <div className="hidden md:block">
                    <span className="text-xs bg-white/5 px-2 py-1 rounded text-gray-400">{call.duration_sec}s</span>
                    <span className="text-xs bg-primary/10 text-primary px-2 py-1 rounded ml-2 uppercase">{call.platform}</span>
                    <span className="text-xs bg-secondary/10 text-secondary px-2 py-1 rounded ml-2 uppercase">{call.language}</span>
                  </div>
                  <div className="hidden lg:block text-sm text-gray-400 max-w-md truncate">
                    {call.summary}
                  </div>
                </div>
                {expandedId === call.id ? <ChevronUp className="w-5 h-5 text-gray-500" /> : <ChevronDown className="w-5 h-5 text-gray-500" />}
              </div>

              {expandedId === call.id && (
                <div className="p-4 border-t border-white/5 bg-black/40">
                  <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-2">AI Summary</h4>
                  <p className="text-sm text-gray-300 mb-6">{call.summary}</p>
                  
                  <h4 className="text-xs font-bold text-gray-500 uppercase tracking-wider mb-2">Full Transcript</h4>
                  <div className="bg-[#0a0a0f] p-4 rounded-lg border border-white/5 font-mono text-xs whitespace-pre-wrap leading-relaxed">
                    {call.full_transcript || 'No transcript recorded.'}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
