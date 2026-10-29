import { X } from 'lucide-react';
import { useModelStore } from '../stores/modelStore';

export default function DownloadProgress() {
  const { downloadQueue, removeDownloadTask } = useModelStore();
  const tasks = Object.values(downloadQueue);

  if (tasks.length === 0) return null;

  return (
    <div className="fixed bottom-6 right-6 w-80 space-y-3 z-50">
      {tasks.map(task => (
        <div key={task.model_id} className="bg-surface border border-white/10 rounded-xl p-4 shadow-2xl">
          <div className="flex justify-between items-center mb-2">
            <span className="text-sm font-medium truncate pr-2">{task.model_id}</span>
            <button onClick={() => removeDownloadTask(task.model_id)} className="text-gray-500 hover:text-white">
              <X className="w-4 h-4" />
            </button>
          </div>
          
          <div className="w-full h-2 bg-black/50 rounded-full overflow-hidden mb-2">
            <div 
              className="h-full bg-gradient-to-r from-primary to-secondary transition-all duration-300"
              style={{ width: `${task.progress}%` }}
            />
          </div>
          
          <div className="flex justify-between text-xs text-gray-400">
            <span>{task.downloaded_mb.toFixed(1)} / {task.total_mb.toFixed(1)} MB</span>
            <span>{task.speed_mbps.toFixed(1)} MB/s • {task.eta_seconds}s</span>
          </div>
        </div>
      ))}
    </div>
  );
}
