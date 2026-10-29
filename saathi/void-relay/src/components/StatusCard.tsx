import React from 'react';

interface StatusCardProps {
  icon: React.ElementType;
  title: string;
  value: string;
  subtitle?: string;
  status: 'active' | 'warning' | 'error' | 'idle';
}

export default function StatusCard({ icon: Icon, title, value, subtitle, status }: StatusCardProps) {
  const getStatusColor = () => {
    switch (status) {
      case 'active': return 'text-success bg-success/10 border-success/30 shadow-[0_0_15px_rgba(34,197,94,0.15)]';
      case 'warning': return 'text-amber-400 bg-amber-400/10 border-amber-400/30';
      case 'error': return 'text-danger bg-danger/10 border-danger/30';
      case 'idle': return 'text-gray-400 bg-white/5 border-white/10';
    }
  };

  const getDotColor = () => {
    switch (status) {
      case 'active': return 'bg-success shadow-[0_0_8px_rgba(34,197,94,0.6)] animate-pulse-slow';
      case 'warning': return 'bg-amber-400';
      case 'error': return 'bg-danger';
      case 'idle': return 'bg-gray-500';
    }
  };

  return (
    <div className={`p-5 rounded-xl border transition-all duration-300 ${getStatusColor()}`}>
      <div className="flex items-start justify-between mb-4">
        <div className={`p-2 rounded-lg bg-black/20`}>
          <Icon className="w-5 h-5" />
        </div>
        <div className={`w-2.5 h-2.5 rounded-full ${getDotColor()}`} />
      </div>
      <div>
        <h3 className="text-sm font-medium text-gray-400 mb-1">{title}</h3>
        <p className="text-lg font-bold truncate">{value}</p>
        {subtitle && <p className="text-xs text-gray-500 mt-1 truncate">{subtitle}</p>}
      </div>
    </div>
  );
}
