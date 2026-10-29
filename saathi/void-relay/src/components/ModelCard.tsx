import { Download, Trash2, CheckCircle2, Server, Clock } from 'lucide-react';
import { ModelInfo } from '../stores/modelStore';

interface ModelCardProps {
  model: ModelInfo;
  isDownloaded: boolean;
  isActive: boolean;
  isDownloading: boolean;
  onDownload: (id: string) => void;
  onDelete: (id: string) => void;
  onSetActive: (id: string) => void;
}

export default function ModelCard({ model, isDownloaded, isActive, isDownloading, onDownload, onDelete, onSetActive }: ModelCardProps) {
  return (
    <div className={`p-4 rounded-xl border transition-all ${isActive ? 'border-primary bg-primary/5' : 'border-white/5 bg-black/20 hover:border-white/10'}`}>
      <div className="flex justify-between items-start mb-3">
        <div>
          <h4 className="font-semibold text-gray-100 flex items-center gap-2">
            {model.name}
            {isActive && <span className="text-[10px] uppercase px-1.5 py-0.5 rounded bg-primary/20 text-primary font-bold tracking-wider">Active</span>}
          </h4>
          <p className="text-xs text-gray-500 mt-1">
            {model.category} • {model.language.join(', ').toUpperCase()} • {model.quantization}
          </p>
        </div>
        <div className="flex gap-2">
          {!isDownloaded && !isDownloading && (
            <button onClick={() => onDownload(model.id)} className="p-1.5 bg-white/5 hover:bg-primary/20 text-gray-400 hover:text-primary rounded-lg transition-colors">
              <Download className="w-4 h-4" />
            </button>
          )}
          {isDownloaded && !isActive && (
            <>
              <button onClick={() => onSetActive(model.id)} className="p-1.5 bg-success/10 hover:bg-success/20 text-success rounded-lg transition-colors" title="Set Active">
                <CheckCircle2 className="w-4 h-4" />
              </button>
              <button onClick={() => onDelete(model.id)} className="p-1.5 bg-danger/10 hover:bg-danger/20 text-danger rounded-lg transition-colors" title="Delete">
                <Trash2 className="w-4 h-4" />
              </button>
            </>
          )}
        </div>
      </div>
      
      <div className="grid grid-cols-2 gap-4 mt-4 text-sm">
        <div className="flex items-center gap-2 text-gray-400">
          <Server className="w-4 h-4" />
          <span>{model.size_mb} MB (Req: {model.min_ram_gb}GB RAM)</span>
        </div>
        <div className="flex items-center gap-2 text-gray-400">
          <Clock className="w-4 h-4" />
          <span>Spd: {model.speed_score}/10 | Qly: {model.quality_score}/10</span>
        </div>
      </div>
    </div>
  );
}
