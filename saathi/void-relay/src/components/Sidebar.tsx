import { LayoutDashboard, History, Box, Settings, Activity } from 'lucide-react';
import { useAppStore } from '../stores/appStore';

export default function Sidebar() {
  const { activeTab, setActiveTab } = useAppStore();

  const navItems = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'history', label: 'Call Archives', icon: History },
    { id: 'models', label: 'Models & Engine', icon: Box },
    { id: 'settings', label: 'Settings', icon: Settings },
  ] as const;

  return (
    <aside className="w-64 glass-panel border-l-0 border-y-0 rounded-none rounded-r-2xl flex flex-col p-6 z-20">
      <div className="flex items-center gap-3 mb-12">
        <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-primary to-secondary flex items-center justify-center shadow-[0_0_15px_rgba(79,70,229,0.5)]">
          <Activity className="w-6 h-6 text-white" />
        </div>
        <div>
          <h1 className="text-xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-gray-400">
            VoidRelay
          </h1>
          <p className="text-xs text-gray-400 font-medium tracking-wider uppercase">Zero-Cost AI</p>
        </div>
      </div>

      <nav className="flex-1 space-y-2">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = activeTab === item.id;
          
          return (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-300 font-medium
                ${isActive 
                  ? 'bg-primary/10 text-primary shadow-[inset_2px_0_0_0_rgba(79,70,229,1)]' 
                  : 'text-gray-400 hover:bg-white/5 hover:text-white'
                }`}
            >
              <Icon className={`w-5 h-5 ${isActive ? 'text-primary' : ''}`} />
              {item.label}
            </button>
          );
        })}
      </nav>

      <div className="mt-auto pt-6 border-t border-white/5">
        <div className="flex items-center gap-2 text-sm text-gray-400">
          <div className="w-2 h-2 rounded-full bg-success shadow-[0_0_8px_rgba(34,197,94,0.6)] animate-pulse-slow" />
          System Online
        </div>
      </div>
    </aside>
  );
}
