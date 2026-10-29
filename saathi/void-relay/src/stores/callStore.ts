import { create } from 'zustand';

export type CallState = 'IDLE' | 'INITIATING' | 'RINGING' | 'CONNECTED' | 'SCREENING' | 'HANDOFF' | 'LISTENING' | 'PROCESSING' | 'SPEAKING' | 'ENDED';

export interface ChatMessage {
  id: string;
  role: 'user' | 'ai' | 'system';
  content: string;
  timestamp: Date;
}

interface CallStore {
  callState: CallState;
  activeNumber: string | null;
  activePhone: 'android' | 'ios' | 'none' | null;
  conversationHistory: ChatMessage[];
  llmStreamContent: string;
  setCallState: (state: CallState) => void;
  setActiveCall: (number: string, phone: 'android' | 'ios') => void;
  addMessage: (message: Omit<ChatMessage, 'id' | 'timestamp'>) => void;
  updateLlmStream: (content: string) => void;
  clearLlmStream: () => void;
  endCall: () => void;
}

export const useCallStore = create<CallStore>((set) => ({
  callState: 'IDLE',
  activeNumber: null,
  activePhone: null,
  conversationHistory: [],
  llmStreamContent: '',
  setCallState: (callState) => set({ callState }),
  setActiveCall: (activeNumber, activePhone) => set({ activeNumber, activePhone, conversationHistory: [] }),
  addMessage: (msg) => set((state) => ({
    conversationHistory: [...state.conversationHistory, { ...msg, id: Math.random().toString(36).substr(2, 9), timestamp: new Date() }]
  })),
  updateLlmStream: (content) => set((state) => ({ llmStreamContent: state.llmStreamContent + content })),
  clearLlmStream: () => set({ llmStreamContent: '' }),
  endCall: () => set({ callState: 'ENDED', activeNumber: null, activePhone: null }),
}));
