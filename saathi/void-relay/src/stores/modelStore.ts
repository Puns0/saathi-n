import { create } from 'zustand';

export interface ModelInfo {
  id: string;
  name: string;
  category: 'STT' | 'TTS' | 'LLM';
  language: string[];
  size_mb: number;
  ram_required_mb: number;
  download_url: string;
  filename: string;
  sha256: string;
  quantization: string;
  min_ram_gb: number;
  quality_score: number;
  speed_score: number;
}

export interface DownloadedModel {
  info: ModelInfo;
  downloaded_mb: number;
  is_verified: boolean;
}

export interface ActiveModelConfig {
  primary_stt_english: string | null;
  primary_stt_indic: string | null;
  primary_tts_english: string | null;
  primary_tts_hindi: string | null;
  primary_tts_kannada: string | null;
  primary_llm: string | null;
}

export interface DownloadTask {
  model_id: string;
  progress: number; // 0-100
  downloaded_mb: number;
  total_mb: number;
  speed_mbps: number;
  eta_seconds: number;
}

interface ModelStore {
  availableModels: ModelInfo[];
  downloadedModels: DownloadedModel[];
  activeConfig: ActiveModelConfig;
  downloadQueue: Record<string, DownloadTask>;
  setAvailableModels: (models: ModelInfo[]) => void;
  setDownloadedModels: (models: DownloadedModel[]) => void;
  setActiveConfig: (config: Partial<ActiveModelConfig>) => void;
  updateDownloadProgress: (task: DownloadTask) => void;
  removeDownloadTask: (model_id: string) => void;
}

export const useModelStore = create<ModelStore>((set) => ({
  availableModels: [],
  downloadedModels: [],
  activeConfig: {
    primary_stt_english: null,
    primary_stt_indic: null,
    primary_tts_english: null,
    primary_tts_hindi: null,
    primary_tts_kannada: null,
    primary_llm: null,
  },
  downloadQueue: {},
  setAvailableModels: (availableModels) => set({ availableModels }),
  setDownloadedModels: (downloadedModels) => set({ downloadedModels }),
  setActiveConfig: (config) => set((state) => ({ activeConfig: { ...state.activeConfig, ...config } })),
  updateDownloadProgress: (task) => set((state) => ({
    downloadQueue: { ...state.downloadQueue, [task.model_id]: task }
  })),
  removeDownloadTask: (model_id) => set((state) => {
    const { [model_id]: _, ...rest } = state.downloadQueue;
    return { downloadQueue: rest };
  }),
}));
