import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from './stores/appStore';
import Dashboard from './pages/Dashboard';
import CallHistory from './pages/CallHistory';
import Models from './pages/Models';
import Settings from './pages/Settings';
import Sidebar from './components/Sidebar';

function App() {
  const { activeTab, addPhone, removePhone, setBtConnected } = useAppStore();

  useEffect(() => {
    // Setup event listeners for backend
    const unlistenPhones = listen('phone_connected', (e: any) => {
      addPhone(e.payload);
    });

    const unlistenDisconn = listen('phone_disconnected', (e: any) => {
      removePhone(e.payload);
    });

    const unlistenBt = listen('bt_status', (e: any) => {
      setBtConnected(e.payload.connected);
    });
    
    // Polling call state (could also use events)
    // ...

    return () => {
      unlistenPhones.then(f => f());
      unlistenDisconn.then(f => f());
      unlistenBt.then(f => f());
    };
  }, []);

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar Navigation */}
      <Sidebar />
      
      {/* Main Content Area */}
      <main className="flex-1 relative overflow-y-auto">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-background to-accent/5 pointer-events-none" />
        
        <div className="relative z-10 p-8 max-w-7xl mx-auto h-full">
          {activeTab === 'dashboard' && <Dashboard />}
          {activeTab === 'history' && <CallHistory />}
          {activeTab === 'models' && <Models />}
          {activeTab === 'settings' && <Settings />}
        </div>
      </main>
    </div>
  );
}

export default App;
