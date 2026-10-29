import { create } from 'zustand';

export type LogCategory = 'ALL' | 'BT' | 'STT' | 'LLM' | 'TTS' | 'CALLS' | 'SYSTEM';
export type LogLevel = 'success' | 'info' | 'warn' | 'error' | 'debug';

export interface LogEntry {
  id: string;
  timestamp: Date;
  category: LogCategory;
  level: LogLevel;
  message: string;
}

interface LogStore {
  entries: LogEntry[];
  filter: LogCategory;
  addLog: (entry: Omit<LogEntry, 'id' | 'timestamp'>) => void;
  setFilter: (filter: LogCategory) => void;
  clearLogs: () => void;
}

const MAX_LOGS = 500;

export const useLogStore = create<LogStore>((set) => ({
  entries: [],
  filter: 'ALL',
  addLog: (entry) => set((state) => {
    const newEntry: LogEntry = {
      ...entry,
      id: Math.random().toString(36).substr(2, 9),
      timestamp: new Date()
    };
    const newEntries = [...state.entries, newEntry];
    if (newEntries.length > MAX_LOGS) {
      newEntries.shift();
    }
    return { entries: newEntries };
  }),
  setFilter: (filter) => set({ filter }),
  clearLogs: () => set({ entries: [] }),
}));
