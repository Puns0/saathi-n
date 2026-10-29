import { create } from 'zustand';

export interface AppConfig {
  ai_mode: 'online' | 'offline' | 'auto';
  groq_api_key: string;
  sarvam_api_key: string;
  default_language: 'auto' | 'en' | 'hi' | 'kn';
  system_prompt: string;
  vad_sensitivity: number;
  silence_timeout_ms: number;
  streaming_tts: boolean;
  user_name: string;
  android_phone_name: string;
  ios_phone_name: string;
  ws_port: number;
  preferred_phone: 'auto' | 'android' | 'ios';
  telegram_bot_token: string;
  telegram_chat_id: string;
}

const defaultConfig: AppConfig = {
  ai_mode: 'auto',
  groq_api_key: '',
  sarvam_api_key: '',
  default_language: 'auto',
  system_prompt: 'You are a voice assistant making/receiving a phone call on behalf of the user. Be concise, professional, and match the caller language.',
  vad_sensitivity: 0.015,
  silence_timeout_ms: 800,
  streaming_tts: true,
  user_name: 'User',
  android_phone_name: 'Android',
  ios_phone_name: 'iPhone',
  ws_port: 7331,
  preferred_phone: 'auto',
  telegram_bot_token: '',
  telegram_chat_id: '',
};

interface ConfigStore {
  config: AppConfig;
  updateConfig: (updates: Partial<AppConfig>) => void;
  setConfig: (config: AppConfig) => void;
}

export const useConfigStore = create<ConfigStore>((set) => ({
  config: defaultConfig,
  updateConfig: (updates) => set((state) => ({ config: { ...state.config, ...updates } })),
  setConfig: (config) => set({ config }),
}));
