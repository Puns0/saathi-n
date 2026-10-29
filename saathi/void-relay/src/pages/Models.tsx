import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useModelStore, ModelInfo } from '../stores/modelStore';
import { Cpu, Database, Settings2 } from 'lucide-react';

import RecommendationCard, { Recommendation } from '../components/RecommendationCard';
import ModelCard from '../components/ModelCard';
import DownloadProgress from '../components/DownloadProgress';

interface DeviceProfile {
  cpu_model: string;
  total_ram_gb: number;
  available_ram_gb: number;
  has_gpu: boolean;
  is_apple_silicon: boolean;
}

export default function Models() {
  const { availableModels, downloadedModels, activeConfig, setAvailableModels, setDownloadedModels, setActiveConfig } = useModelStore();
  const [profile, setProfile] = useState<DeviceProfile | null>(null);
  const [recommendation, setRecommendation] = useState<Recommendation | null>(null);
  const [loadingRec, setLoadingRec] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const prof = await invoke<DeviceProfile>('get_device_profile');
      setProfile(prof);
      const allModels = await invoke<ModelInfo[]>('list_available_models');
      setAvailableModels(allModels);
      const dl = await invoke<any[]>('list_downloaded_models');
      setDownloadedModels(dl);
      const act = await invoke<any>('get_active_models');
      setActiveConfig(act);
    } catch (e) {
      console.error(e);
    }
  };

  const getRecommendation = async () => {
    setLoadingRec(true);
    try {
      const rec = await invoke<Recommendation>('get_recommendations');
      setRecommendation(rec);
    } catch (e) {
      console.error(e);
      alert('Failed to get recommendation. Check Groq API key.');
    }
    setLoadingRec(false);
  };

  const handleDownload = async (id: string) => {
    try {
      await invoke('download_model', { modelId: id });
    } catch (e) {
      console.error(e);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm(`Delete model ${id}?`)) {
      try {
        await invoke('delete_model', { modelId: id });
        loadData();
      } catch (e) { console.error(e); }
    }
  };

  const handleSetActive = async (slotId: string, modelId: string) => {
    try {
      await invoke('switch_model', { slot: slotId, modelId });
      loadData();
    } catch (e) { console.error(e); }
  };

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20 relative">
      <header className="flex justify-between items-end">
        <div>
          <h2 className="text-3xl font-bold tracking-tight mb-2">Engine Architecture</h2>
          <p className="text-gray-400">Manage offline models and active slots.</p>
        </div>
        <button onClick={getRecommendation} disabled={loadingRec} className="btn-primary flex items-center gap-2">
          <Settings2 className="w-4 h-4" /> {loadingRec ? 'Analyzing...' : 'Auto-Optimize'}
        </button>
      </header>

      {/* Hardware Profiler */}
      {profile && (
        <div className="glass-panel p-6 border-l-4 border-l-primary relative overflow-hidden">
          <div className="absolute right-0 top-0 p-8 opacity-5">
            <Cpu className="w-32 h-32" />
          </div>
          <h3 className="font-semibold text-lg mb-4 flex items-center gap-2 text-primary">
            <Cpu className="w-5 h-5" /> Hardware Profile
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            <div><div className="text-sm text-gray-400 mb-1">Processor</div><div className="font-medium truncate">{profile.cpu_model}</div></div>
            <div><div className="text-sm text-gray-400 mb-1">Total RAM</div><div className="font-medium">{profile.total_ram_gb.toFixed(1)} GB</div></div>
            <div><div className="text-sm text-gray-400 mb-1">Available RAM</div><div className="font-medium text-success">{profile.available_ram_gb.toFixed(1)} GB</div></div>
            <div><div className="text-sm text-gray-400 mb-1">Acceleration</div><div className="font-medium">{profile.is_apple_silicon ? 'Apple Neural Engine' : profile.has_gpu ? 'GPU' : 'CPU Only'}</div></div>
          </div>
        </div>
      )}

      {/* Recommendation */}
      {recommendation && (
        <RecommendationCard 
          recommendation={recommendation} 
          deviceInfo={profile?.cpu_model || 'Unknown'} 
          onAccept={() => {
            // Initiate downloads for all
            [recommendation.primary_llm, recommendation.primary_stt_english, recommendation.primary_stt_indic, recommendation.primary_tts_english, recommendation.primary_tts_hindi, recommendation.primary_tts_kannada]
              .forEach(id => { if (id) handleDownload(id); });
          }} 
          onCustomize={() => setRecommendation(null)} 
        />
      )}

      {/* Model Registry List */}
      <div className="space-y-6">
        <h3 className="font-semibold text-xl flex items-center gap-2">
          <Database className="w-5 h-5 text-secondary" /> Offline Model Registry
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {['STT', 'LLM', 'TTS'].map(category => (
            <div key={category} className="glass-panel p-6">
              <h4 className="font-medium text-lg mb-4 pb-2 border-b border-white/10">{category} Models</h4>
              <div className="space-y-3 max-h-[500px] overflow-y-auto pr-2">
                {availableModels.filter(m => m.category === category).map(model => {
                  const isDownloaded = downloadedModels.some(d => d.info.id === model.id);
                  const isActive = Object.values(activeConfig).includes(model.id);
                  
                  return (
                    <ModelCard 
                      key={model.id}
                      model={model}
                      isDownloaded={isDownloaded}
                      isActive={isActive}
                      isDownloading={false} // hook up to download queue
                      onDownload={handleDownload}
                      onDelete={handleDelete}
                      onSetActive={(id) => handleSetActive(category.toLowerCase(), id)} // simplification
                    />
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      </div>

      <DownloadProgress />
    </div>
  );
}
