import { Cpu, CheckCircle, Settings2 } from 'lucide-react';

export interface Recommendation {
  primary_stt_english: string;
  primary_stt_indic: string;
  primary_tts_english: string;
  primary_tts_hindi: string;
  primary_tts_kannada: string;
  primary_llm: string;
  reasoning: string;
  warnings: string[];
  total_ram_required_gb: number;
  total_disk_required_gb: number;
  confidence: 'high' | 'medium' | 'low';
}

interface RecommendationCardProps {
  recommendation: Recommendation;
  deviceInfo: string;
  onAccept: () => void;
  onCustomize: () => void;
}

export default function RecommendationCard({ recommendation, deviceInfo, onAccept, onCustomize }: RecommendationCardProps) {
  return (
    <div className="glass-panel p-6 border border-primary/30 relative overflow-hidden">
      <div className="absolute top-0 right-0 w-64 h-64 bg-primary/10 rounded-full blur-3xl -mr-20 -mt-20 pointer-events-none" />
      
      <div className="flex items-center gap-3 mb-6 relative z-10">
        <div className="p-2 bg-primary/20 rounded-lg text-primary">
          <Cpu className="w-6 h-6" />
        </div>
        <div>
          <h3 className="text-xl font-bold">AI Recommended Configuration</h3>
          <p className="text-sm text-gray-400">Based on: {deviceInfo}</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 relative z-10">
        <div className="space-y-4">
          <div className="bg-black/30 p-4 rounded-xl border border-white/5">
            <h4 className="text-sm font-semibold text-gray-400 mb-2 uppercase tracking-wider">Model Selection</h4>
            <ul className="space-y-2 text-sm">
              <li className="flex justify-between">
                <span className="text-gray-500">STT (EN):</span>
                <span className="font-medium text-white truncate max-w-[150px]">{recommendation.primary_stt_english}</span>
              </li>
              <li className="flex justify-between">
                <span className="text-gray-500">STT (Indic):</span>
                <span className="font-medium text-white truncate max-w-[150px]">{recommendation.primary_stt_indic}</span>
              </li>
              <li className="flex justify-between">
                <span className="text-gray-500">LLM:</span>
                <span className="font-medium text-white truncate max-w-[150px]">{recommendation.primary_llm}</span>
              </li>
              <li className="flex justify-between">
                <span className="text-gray-500">TTS (EN/HI/KN):</span>
                <span className="font-medium text-white truncate max-w-[150px]">{recommendation.primary_tts_english}</span>
              </li>
            </ul>
          </div>
        </div>

        <div className="space-y-4">
          <div className="bg-black/30 p-4 rounded-xl border border-white/5 h-full">
            <h4 className="text-sm font-semibold text-gray-400 mb-2 uppercase tracking-wider">Reasoning</h4>
            <p className="text-sm text-gray-300 leading-relaxed italic">"{recommendation.reasoning}"</p>
            
            <div className="mt-4 flex items-center gap-4 text-xs font-mono text-gray-400">
              <span className="bg-black/50 px-2 py-1 rounded">RAM: {recommendation.total_ram_required_gb.toFixed(1)}GB</span>
              <span className="bg-black/50 px-2 py-1 rounded">Disk: {recommendation.total_disk_required_gb.toFixed(1)}GB</span>
              <span className={`px-2 py-1 rounded font-bold uppercase ${recommendation.confidence === 'high' ? 'text-success bg-success/10' : 'text-amber-400 bg-amber-400/10'}`}>
                {recommendation.confidence} Confidence
              </span>
            </div>
          </div>
        </div>
      </div>

      <div className="flex gap-4 mt-6 relative z-10">
        <button onClick={onAccept} className="btn-primary flex-1 flex justify-center items-center gap-2 py-3">
          <CheckCircle className="w-5 h-5" /> Accept & Download All
        </button>
        <button onClick={onCustomize} className="btn-secondary flex-1 flex justify-center items-center gap-2 py-3">
          <Settings2 className="w-5 h-5" /> Customize Manually
        </button>
      </div>
    </div>
  );
}
