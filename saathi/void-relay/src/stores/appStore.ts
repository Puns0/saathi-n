import { create } from 'zustand';

export type CallState = 'Idle' | 'Initiating' | 'Ringing' | 'Connected' | 'Listening' | 'Processing' | 'Speaking' | 'Ended';

export interface PhoneDevice {
  platform: string;
  name: string;
  ip: string;
}

interface AppState {
  activeTab: 'dashboard' | 'history' | 'models' | 'settings' | 'diagnostics';
  setActiveTab: (tab: 'dashboard' | 'history' | 'models' | 'settings' | 'diagnostics') => void;
  
  phones: PhoneDevice[];
  addPhone: (phone: PhoneDevice) => void;
  removePhone: (platform: string) => void;
  
  btConnected: boolean;
  setBtConnected: (connected: boolean) => void;
  
  callState: CallState;
  setCallState: (state: CallState) => void;
  
  onlineMode: boolean;
  setOnlineMode: (online: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  activeTab: 'dashboard',
  setActiveTab: (tab) => set({ activeTab: tab }),
  
  phones: [],
  addPhone: (phone) => set((state) => {
    const exists = state.phones.find(p => p.platform === phone.platform);
    if (exists) {
      return { phones: state.phones.map(p => p.platform === phone.platform ? phone : p) };
    }
    return { phones: [...state.phones, phone] };
  }),
  removePhone: (platform) => set((state) => ({
    phones: state.phones.filter(p => p.platform !== platform)
  })),
  
  btConnected: false,
  setBtConnected: (connected) => set({ btConnected: connected }),
  
  callState: 'Idle',
  setCallState: (callState) => set({ callState }),
  
  onlineMode: false,
  setOnlineMode: (onlineMode) => set({ onlineMode }),
}));
