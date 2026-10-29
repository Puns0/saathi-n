import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

interface AudioVisualizerProps {
  isActive: boolean;
}

export default function AudioVisualizer({ isActive }: AudioVisualizerProps) {
  const [levels, setLevels] = useState<number[]>(Array(32).fill(0));

  useEffect(() => {
    if (!isActive) {
      setLevels(Array(32).fill(0));
      return;
    }

    // Fallback animation if no real events are coming
    const interval = setInterval(() => {
      setLevels(prev => prev.map(() => Math.random() * 100));
    }, 100);

    // Listen for real audio levels from Tauri
    const unlisten = listen<{ levels: number[] }>('audio_levels', (event) => {
      setLevels(event.payload.levels);
    });

    return () => {
      clearInterval(interval);
      unlisten.then(f => f());
    };
  }, [isActive]);

  return (
    <div className="flex items-center justify-center h-32 gap-1 w-full bg-black/20 rounded-xl border border-white/5 p-4 overflow-hidden">
      {levels.map((level, i) => (
        <div
          key={i}
          className="w-full max-w-[8px] bg-primary rounded-t transition-all duration-75 ease-out"
          style={{
            height: `${Math.max(5, level)}%`,
            opacity: isActive ? 0.7 + (level / 300) : 0.2,
            boxShadow: isActive && level > 50 ? '0 0 10px rgba(79, 70, 229, 0.5)' : 'none'
          }}
        />
      ))}
    </div>
  );
}
