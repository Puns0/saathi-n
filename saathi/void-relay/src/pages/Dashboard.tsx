import { useEffect, useState } from 'react';
import { useAppStore } from '../stores/appStore';
import { useCallStore } from '../stores/callStore';
import { useConfigStore } from '../stores/configStore';
import { Smartphone, Bluetooth, Activity, Database, Globe, WifiOff } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import StatusCard from '../components/StatusCard';
import AudioVisualizer from '../components/AudioVisualizer';
import ConversationView from '../components/ConversationView';
import LogTerminal from '../components/LogTerminal';

export default function Dashboard() {
  const { phones, btConnected } = useAppStore();
  const { 
    callState, 
    activeNumber, 
    setCallState, 
    setActiveCall, 
    addMessage, 
    updateLlmStream, 
    clearLlmStream 
  } = useCallStore();
  const { config, updateConfig } = useConfigStore();
  const [dialNumber, setDialNumber] = useState('');

  useEffect(() => {
    const unlistenCallState = listen<{ state: any }>('call_state_changed', (e) => {
      console.log('Call state changed:', e.payload.state);
      setCallState(e.payload.state);
    });

    const unlistenCallStarted = listen<{ number: string; platform: any }>('call_started', (e) => {
      console.log('Call started:', e.payload);
      setActiveCall(e.payload.number, e.payload.platform === 'none' ? 'none' : e.payload.platform);
    });

    const unlistenSttTranscript = listen<{ text: string }>('stt_transcript', (e) => {
      console.log('STT transcript:', e.payload.text);
      addMessage({ role: 'user', content: e.payload.text });
    });

    const unlistenLlmToken = listen<{ token: string }>('llm_token', (e) => {
      updateLlmStream(e.payload.token);
    });

    const unlistenLlmFinished = listen<{ text: string }>('llm_finished', (e) => {
      console.log('LLM finished:', e.payload.text);
      clearLlmStream();
      addMessage({ role: 'ai', content: e.payload.text });
    });

    return () => {
      unlistenCallState.then(f => f());
      unlistenCallStarted.then(f => f());
      unlistenSttTranscript.then(f => f());
      unlistenLlmToken.then(f => f());
      unlistenLlmFinished.then(f => f());
    };
  }, []);

  const handleDial = async () => {
    if (!dialNumber) return;
    try {
      await invoke('initiate_call', { number: dialNumber, platformHint: null });
    } catch (e) {
      console.error(e);
    }
  };

  const handleHangup = async () => {
    try {
      await invoke('hangup_call');
    } catch (e) {
      console.error(e);
    }
  };

  const cycleMode = () => {
    const modes: ('online' | 'offline' | 'auto')[] = ['auto', 'online', 'offline'];
    const currentIdx = modes.indexOf(config.ai_mode);
    const nextMode = modes[(currentIdx + 1) % modes.length];
    updateConfig({ ai_mode: nextMode });
    invoke('save_settings', { settings: { ...config, ai_mode: nextMode } });
  };

  return (
    <div className="h-full flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-8">
      <header>
        <h2 className="text-3xl font-bold tracking-tight mb-2">Command Center</h2>
        <p className="text-gray-400">Monitor and control your AI voice agents across all devices.</p>
      </header>

      {/* Top row: Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatusCard 
          icon={Smartphone} 
          title="Active Relays" 
          value={`${phones.length} Connected`}
          subtitle={phones.map(p => p.platform).join(', ')}
          status={phones.length > 0 ? 'active' : 'idle'} 
        />
        <StatusCard 
          icon={Bluetooth} 
          title="Bluetooth Audio" 
          value={btConnected ? 'HFP Active' : 'Disconnected'}
          subtitle={btConnected ? 'HFP Audio Active' : 'Awaiting call...'}
          status={btConnected ? 'active' : 'idle'} 
        />
        <div className="p-5 rounded-xl border border-white/10 bg-black/20 hover:bg-white/5 transition-colors cursor-pointer" onClick={cycleMode}>
          <div className="flex items-start justify-between mb-4">
            <div className={`p-2 rounded-lg ${config.ai_mode === 'online' ? 'bg-secondary/20 text-secondary' : config.ai_mode === 'offline' ? 'bg-danger/20 text-danger' : 'bg-primary/20 text-primary'}`}>
              {config.ai_mode === 'online' ? <Globe className="w-5 h-5" /> : config.ai_mode === 'offline' ? <WifiOff className="w-5 h-5" /> : <Database className="w-5 h-5" />}
            </div>
          </div>
          <h3 className="text-sm font-medium text-gray-400 mb-1">Engine Mode</h3>
          <p className="text-lg font-bold capitalize">{config.ai_mode}</p>
        </div>
        <StatusCard 
          icon={Activity} 
          title="Live Call Status" 
          value={callState}
          subtitle={activeNumber ? `Calling ${activeNumber}` : ''}
          status={callState === 'IDLE' || callState === 'ENDED' ? 'idle' : callState === 'CONNECTED' ? 'active' : 'warning'} 
        />
      </div>

      {/* Main split: Visualizer/Chat & Logs */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-[400px]">
        
        {/* Left Column: Call Control & Visualizer */}
        <div className="lg:col-span-2 flex flex-col gap-6">
          <div className="glass-panel p-6 flex flex-col relative overflow-hidden">
            <div className="absolute -right-20 -top-20 w-64 h-64 bg-primary/10 rounded-full blur-3xl pointer-events-none" />
            
            <div className="flex justify-between items-center mb-6 z-10">
              <h3 className="text-xl font-semibold">Live Session</h3>
              {callState !== 'IDLE' && callState !== 'ENDED' && (
                <div className="flex gap-3">
                  <button onClick={handleHangup} className="px-4 py-2 bg-danger/20 text-danger border border-danger/30 rounded-lg hover:bg-danger/30 transition-colors font-semibold">
                    End Call
                  </button>
                </div>
              )}
            </div>

            <AudioVisualizer isActive={callState === 'LISTENING' || callState === 'PROCESSING' || callState === 'SPEAKING'} />
            
            <div className="mt-6 flex-1 min-h-[250px]">
              <ConversationView />
            </div>
            
            {/* Dialer (only when idle) */}
            {(callState === 'IDLE' || callState === 'ENDED') && (
              <div className="mt-6 flex gap-3 z-10">
                <input 
                  type="text" 
                  placeholder="Enter phone number to dial..." 
                  className="input-field flex-1"
                  value={dialNumber}
                  onChange={e => setDialNumber(e.target.value)}
                />
                <button onClick={handleDial} className="btn-primary px-8">Dial</button>
              </div>
            )}
          </div>
        </div>

        {/* Right Column: Logs */}
        <div className="h-full">
          <LogTerminal />
        </div>
      </div>
    </div>
  );
}
