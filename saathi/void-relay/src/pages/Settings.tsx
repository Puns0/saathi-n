import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Save, Key, Radio, Globe, Shield, Mic, Volume2, Smartphone, ArrowRight, Info, Sparkles, Zap } from 'lucide-react';

export default function Settings() {
  const [settings, setSettings] = useState<any>(null);
  const [saving, setSaving] = useState(false);
  const [audioDevices, setAudioDevices] = useState<{name: string, kind: string}[]>([]);
  const [routingMode, setRoutingMode] = useState<'standard' | 'single_cable' | 'dual_cable'>('standard');
  const [routingCall, setRoutingCall] = useState(false);

  useEffect(() => {
    if (settings) {
      const inputLower = settings.audio_input_device.toLowerCase();
      const outputLower = settings.audio_output_device.toLowerCase();
      if (inputLower.includes('cable-a') || outputLower.includes('cable-b') || inputLower.includes('voicemeeter') || outputLower.includes('voicemeeter')) {
        setRoutingMode('dual_cable');
      } else if (inputLower.includes('cable') || outputLower.includes('cable')) {
        setRoutingMode('single_cable');
      } else {
        setRoutingMode('standard');
      }
    }
  }, [settings]);

  const handleAutoConfigureSingleCable = () => {
    const inputCable = audioDevices.find(d => d.kind === 'input' && d.name.toLowerCase().includes('cable'));
    const outputCable = audioDevices.find(d => d.kind === 'output' && d.name.toLowerCase().includes('cable'));

    let newSettings = { ...settings };
    let found = false;

    if (inputCable) {
      newSettings.audio_input_device = inputCable.name;
      found = true;
    }
    if (outputCable) {
      newSettings.audio_output_device = outputCable.name;
      found = true;
    }

    if (found) {
      setSettings(newSettings);
      setRoutingMode('single_cable');
      alert(`Auto-configured Single VB-Cable!\nInput (SAATHI Mic In): ${newSettings.audio_input_device || 'System Default'}\nOutput (AI Voice Out): ${newSettings.audio_output_device || 'System Default'}`);
    } else {
      alert("No VB-Cable devices detected. Please make sure VB-Cable is installed and restart the app.");
    }
  };

  const handleAutoConfigureDualCable = () => {
    const inputCableA = audioDevices.find(d => d.kind === 'input' && d.name.toLowerCase().includes('cable-a'));
    const outputCableB = audioDevices.find(d => d.kind === 'output' && d.name.toLowerCase().includes('cable-b'));

    let newSettings = { ...settings };
    let found = false;

    if (inputCableA) {
      newSettings.audio_input_device = inputCableA.name;
      found = true;
    }
    if (outputCableB) {
      newSettings.audio_output_device = outputCableB.name;
      found = true;
    }

    if (found) {
      setSettings(newSettings);
      setRoutingMode('dual_cable');
      alert(`Auto-configured Dual VB-Cable!\nInput (SAATHI Mic In): ${newSettings.audio_input_device || 'System Default'}\nOutput (AI Voice Out): ${newSettings.audio_output_device || 'System Default'}`);
    } else {
      // Fall back to standard cables
      const inputCable = audioDevices.find(d => d.kind === 'input' && d.name.toLowerCase().includes('cable'));
      const outputCable = audioDevices.find(d => d.kind === 'output' && d.name.toLowerCase().includes('cable'));
      if (inputCable || outputCable) {
        if (inputCable) newSettings.audio_input_device = inputCable.name;
        if (outputCable) newSettings.audio_output_device = outputCable.name;
        setSettings(newSettings);
        setRoutingMode('dual_cable');
        alert(`Auto-configured standard VB-Cable as fallback!\nInput: ${newSettings.audio_input_device}\nOutput: ${newSettings.audio_output_device}`);
      } else {
        alert("No VB-Cable (or Cable A/B) devices detected. Please ensure VB-Cable drivers are installed.");
      }
    }
  };

  const handleAutoConfigureVoicemeeter = () => {
    const inputVM = audioDevices.find(d => d.kind === 'input' && d.name.toLowerCase().includes('voicemeeter') && d.name.toLowerCase().includes('b1'));
    const outputVM = audioDevices.find(d => d.kind === 'output' && d.name.toLowerCase().includes('voicemeeter') && d.name.toLowerCase().includes('aux'));

    let newSettings = { ...settings };
    let found = false;

    if (inputVM) {
      newSettings.audio_input_device = inputVM.name;
      found = true;
    }
    if (outputVM) {
      newSettings.audio_output_device = outputVM.name;
      found = true;
    }

    if (found) {
      setSettings(newSettings);
      setRoutingMode('dual_cable');
      alert(`Auto-configured Voicemeeter Dual Mode!\nInput (SAATHI Mic In): ${newSettings.audio_input_device}\nOutput (AI Voice Out): ${newSettings.audio_output_device}\n\nDon't forget to:\n1. Map Phone Link's Input/Output to Voicemeeter in Windows Sound Mixer.\n2. Disable A1 on Voicemeeter Input strip so you don't hear caller audio out loud!`);
    } else {
      // General search for any voicemeeter
      const generalInput = audioDevices.find(d => d.kind === 'input' && d.name.toLowerCase().includes('voicemeeter'));
      const generalOutput = audioDevices.find(d => d.kind === 'output' && d.name.toLowerCase().includes('voicemeeter'));
      if (generalInput || generalOutput) {
        if (generalInput) newSettings.audio_input_device = generalInput.name;
        if (generalOutput) newSettings.audio_output_device = generalOutput.name;
        setSettings(newSettings);
        setRoutingMode('dual_cable');
        alert(`Auto-configured Voicemeeter Fallback!\nInput: ${newSettings.audio_input_device}\nOutput: ${newSettings.audio_output_device}`);
      } else {
        alert("No Voicemeeter devices detected. Please make sure Voicemeeter is running and restart the app.");
      }
    }
  };

  const handleAutoRouteCall = async () => {
    setRoutingCall(true);
    try {
      const res = await invoke<string>('auto_route_audio');
      alert(`⚡ 1-Click Auto-Routing Success!\n\nDetails:\n${res}`);
    } catch (e) {
      alert(`Failed to auto-route call audio:\n${e}`);
    } finally {
      setRoutingCall(false);
    }
  };

  useEffect(() => {
    async function load() {
      try {
        const s = await invoke('get_settings');
        setSettings(s);
        const devs = await invoke<{name: string, kind: string}[]>('list_audio_devices');
        setAudioDevices(devs);
      } catch (e) {
        console.error(e);
      }
    }
    load();
  }, []);

  const handleChange = (field: string, value: any) => {
    setSettings({ ...settings, [field]: value });
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await invoke('save_settings', { settings });
      // Toast success
    } catch (e) {
      console.error(e);
    }
    setTimeout(() => setSaving(false), 500);
  };

  if (!settings) return null;

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
      <header className="flex justify-between items-end">
        <div>
          <h2 className="text-3xl font-bold tracking-tight mb-2">System Settings</h2>
          <p className="text-gray-400">Configure APIs, behavior, and preferences.</p>
        </div>
        <button 
          onClick={handleSave} 
          disabled={saving}
          className="btn-primary flex items-center gap-2"
        >
          <Save className="w-4 h-4" /> {saving ? 'Saving...' : 'Save Configuration'}
        </button>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* API Keys */}
        <div className="glass-panel p-6 space-y-6">
          <h3 className="font-semibold text-lg flex items-center gap-2 text-primary border-b border-white/10 pb-3">
            <Key className="w-5 h-5" /> API Keys (Online Mode)
          </h3>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1">Groq API Key (LLM)</label>
              <input 
                type="password" 
                className="input-field"
                value={settings.groq_api_key}
                onChange={e => handleChange('groq_api_key', e.target.value)}
                placeholder="gsk_..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1">Sarvam API Key (Indic STT/TTS)</label>
              <input 
                type="password" 
                className="input-field"
                value={settings.sarvam_api_key}
                onChange={e => handleChange('sarvam_api_key', e.target.value)}
                placeholder="Enter Sarvam API Key"
              />
            </div>
          </div>
        </div>

        {/* Engine Behavior */}
        <div className="glass-panel p-6 space-y-6">
          <h3 className="font-semibold text-lg flex items-center gap-2 text-secondary border-b border-white/10 pb-3">
            <Globe className="w-5 h-5" /> Engine Behavior
          </h3>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1">Processing Mode</label>
              <select 
                className="input-field appearance-none"
                value={settings.ai_mode}
                onChange={e => handleChange('ai_mode', e.target.value)}
              >
                <option value="auto">Auto (Online if available, fallback to Offline)</option>
                <option value="online">Strict Online (High Quality, Lowest Latency)</option>
                <option value="offline">Strict Offline (Privacy, No API Costs)</option>
              </select>
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1">Agent System Prompt</label>
              <textarea 
                className="input-field min-h-[100px] resize-y"
                value={settings.system_prompt}
                onChange={e => handleChange('system_prompt', e.target.value)}
              />
            </div>
          </div>
        </div>

        {/* Audio / VAD */}
        <div className="glass-panel p-6 space-y-6">
          <h3 className="font-semibold text-lg flex items-center gap-2 text-accent border-b border-white/10 pb-3">
            <Radio className="w-5 h-5" /> Audio & VAD
          </h3>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1 flex justify-between">
                <span>VAD Sensitivity (Threshold)</span>
                <span className="text-accent">{settings.vad_sensitivity}</span>
              </label>
              <input 
                type="range" min="0.001" max="0.1" step="0.001"
                className="w-full accent-accent"
                value={settings.vad_sensitivity}
                onChange={e => handleChange('vad_sensitivity', parseFloat(e.target.value))}
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1 flex justify-between">
                <span>Silence Timeout (ms)</span>
                <span className="text-accent">{settings.silence_timeout_ms}ms</span>
              </label>
              <input 
                type="range" min="300" max="2000" step="50"
                className="w-full accent-accent"
                value={settings.silence_timeout_ms}
                onChange={e => handleChange('silence_timeout_ms', parseInt(e.target.value))}
              />
              <p className="text-xs text-gray-500 mt-1">Lower values respond faster but might cut off slow speakers.</p>
            </div>
          </div>
        </div>

        {/* Audio Device Selection */}
        <div className="glass-panel p-6 space-y-6 lg:col-span-2 relative overflow-hidden group">
          {/* Subtle background glow */}
          <div className="absolute top-0 right-0 w-64 h-64 bg-green-500/5 rounded-full blur-3xl -z-10 group-hover:bg-green-500/10 transition-colors duration-500" />
          
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 border-b border-white/10 pb-3">
            <h3 className="font-semibold text-lg flex items-center gap-2 text-green-400">
              <Mic className="w-5 h-5 animate-pulse" /> Audio & Call Routing Devices
              <span className="text-xs font-normal text-gray-500 bg-green-400/10 px-2 py-0.5 rounded-full flex items-center gap-1">
                <Sparkles className="w-3 h-3 text-yellow-400" /> Premium Routing
              </span>
            </h3>
            
            {/* Routing Mode Selector Tabs */}
            <div className="flex bg-black/40 p-1 rounded-lg border border-white/5 text-xs">
              <button
                type="button"
                onClick={() => setRoutingMode('standard')}
                className={`px-3 py-1.5 rounded-md transition-all duration-300 ${routingMode === 'standard' ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'text-gray-400 hover:text-white'}`}
              >
                PC Speakers & Mic
              </button>
              <button
                type="button"
                onClick={() => setRoutingMode('single_cable')}
                className={`px-3 py-1.5 rounded-md transition-all duration-300 ${routingMode === 'single_cable' ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'text-gray-400 hover:text-white'}`}
              >
                Phone Link (Single Cable)
              </button>
              <button
                type="button"
                onClick={() => setRoutingMode('dual_cable')}
                className={`px-3 py-1.5 rounded-md transition-all duration-300 ${routingMode === 'dual_cable' ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'text-gray-400 hover:text-white'}`}
              >
                Phone Link (Dual Cable)
              </button>
            </div>
          </div>

          {/* Interactive Visual Audio Routing Diagram */}
          <div className="bg-black/35 rounded-xl border border-white/5 p-5 relative">
            <h4 className="text-sm font-semibold text-gray-300 mb-4 flex items-center gap-1.5">
              <Info className="w-4 h-4 text-blue-400" /> Active Audio Path Visualization
            </h4>

            {routingMode === 'standard' && (
              <div className="flex flex-col md:flex-row items-center justify-around gap-6 py-4 animate-in fade-in zoom-in-95 duration-300">
                <div className="flex flex-col items-center gap-2 p-3 bg-white/5 rounded-lg border border-white/5 w-44 text-center">
                  <Smartphone className="w-8 h-8 text-gray-400" />
                  <span className="text-xs font-semibold">Phone Caller</span>
                  <span className="text-[10px] text-gray-500">Auto-Answer Active</span>
                </div>
                <div className="flex items-center gap-1.5 text-gray-500">
                  <ArrowRight className="w-5 h-5 text-green-400 animate-bounce-horizontal" />
                  <span className="text-[10px] font-mono tracking-wider bg-white/5 px-2 py-0.5 rounded">WEB SOCKET</span>
                  <ArrowRight className="w-5 h-5 text-green-400 animate-bounce-horizontal" />
                </div>
                <div className="flex flex-col items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-lg w-44 text-center shadow-lg shadow-green-500/5">
                  <Mic className="w-8 h-8 text-green-400" />
                  <span className="text-xs font-semibold text-green-300">SAATHI (PC App)</span>
                  <span className="text-[10px] text-green-400/80">Using system speakers & mic</span>
                </div>
              </div>
            )}

            {routingMode === 'single_cable' && (
              <div className="flex flex-col md:flex-row items-center justify-around gap-6 py-4 animate-in fade-in zoom-in-95 duration-300">
                <div className="flex flex-col items-center gap-2 p-3 bg-white/5 rounded-lg border border-white/5 w-36 text-center">
                  <Smartphone className="w-7 h-7 text-gray-400" />
                  <span className="text-xs font-semibold">Phone Call</span>
                </div>
                
                <div className="flex flex-col items-center gap-1">
                  <div className="flex items-center gap-1.5">
                    <span className="text-[9px] text-blue-400 font-mono">Speaker Audio</span>
                    <ArrowRight className="w-4 h-4 text-blue-400" />
                  </div>
                  <div className="p-2 bg-blue-500/10 border border-blue-500/20 rounded-md text-[11px] font-semibold text-blue-300 w-28 text-center">
                    CABLE Input
                  </div>
                  <div className="flex items-center gap-1.5">
                    <ArrowRight className="w-4 h-4 text-emerald-400" />
                    <span className="text-[9px] text-emerald-400 font-mono">Virtual Mic</span>
                  </div>
                </div>

                <div className="flex flex-col items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-lg w-36 text-center shadow-lg">
                  <Radio className="w-7 h-7 text-green-400" />
                  <span className="text-xs font-semibold text-green-300">SAATHI Agent</span>
                </div>

                <div className="flex flex-col items-center gap-1">
                  <div className="flex items-center gap-1.5">
                    <ArrowRight className="w-4 h-4 text-indigo-400" />
                    <span className="text-[9px] text-indigo-400 font-mono">Listen/Record</span>
                  </div>
                  <div className="p-2 bg-indigo-500/10 border border-indigo-500/20 rounded-md text-[11px] font-semibold text-indigo-300 w-28 text-center">
                    CABLE Output
                  </div>
                  <div className="flex items-center gap-1.5">
                    <span className="text-[9px] text-indigo-400 font-mono">App Capture</span>
                    <ArrowRight className="w-4 h-4 text-indigo-400" />
                  </div>
                </div>

                <div className="flex flex-col items-center gap-2 p-3 bg-white/5 rounded-lg border border-white/5 w-36 text-center">
                  <Volume2 className="w-7 h-7 text-gray-400" />
                  <span className="text-xs font-semibold">Phone Link App</span>
                </div>
              </div>
            )}

            {routingMode === 'dual_cable' && (
              <div className="flex flex-col md:flex-row items-center justify-around gap-6 py-4 animate-in fade-in zoom-in-95 duration-300">
                <div className="flex flex-col items-center gap-2 p-3 bg-white/5 rounded-lg border border-white/5 w-36 text-center">
                  <Smartphone className="w-7 h-7 text-gray-400" />
                  <span className="text-xs font-semibold">Phone Link Call</span>
                </div>

                <div className="flex flex-col items-center gap-1">
                  <div className="flex items-center gap-1.5 text-cyan-400">
                    <span className="text-[9px] font-mono">Call Out</span>
                    <ArrowRight className="w-4 h-4" />
                  </div>
                  <div className="p-2 bg-cyan-500/10 border border-cyan-500/20 rounded-md text-[11px] font-semibold text-cyan-300 w-28 text-center">
                    CABLE-A Input
                  </div>
                  <div className="flex items-center gap-1.5 text-cyan-400">
                    <ArrowRight className="w-4 h-4" />
                    <span className="text-[9px] font-mono">PC App In</span>
                  </div>
                </div>

                <div className="flex flex-col items-center gap-2 p-3 bg-green-500/10 border border-green-500/20 rounded-lg w-36 text-center shadow-lg">
                  <Radio className="w-7 h-7 text-green-400 animate-pulse" />
                  <span className="text-xs font-semibold text-green-300">SAATHI Agent</span>
                </div>

                <div className="flex flex-col items-center gap-1">
                  <div className="flex items-center gap-1.5 text-purple-400">
                    <span className="text-[9px] font-mono">AI Voice Play</span>
                    <ArrowRight className="w-4 h-4" />
                  </div>
                  <div className="p-2 bg-purple-500/10 border border-purple-500/20 rounded-md text-[11px] font-semibold text-purple-300 w-28 text-center">
                    CABLE-B Input
                  </div>
                  <div className="flex items-center gap-1.5 text-purple-400">
                    <ArrowRight className="w-4 h-4" />
                    <span className="text-[9px] font-mono">Call Mic In</span>
                  </div>
                </div>

                <div className="flex flex-col items-center gap-2 p-3 bg-white/5 rounded-lg border border-white/5 w-36 text-center">
                  <Volume2 className="w-7 h-7 text-gray-400" />
                  <span className="text-xs font-semibold">Caller Speaker</span>
                </div>
              </div>
            )}
          </div>

          {/* Quick Configurations Actions */}
          <div className="flex flex-wrap gap-3 bg-white/5 border border-white/5 p-4 rounded-xl items-center">
            <span className="text-xs font-medium text-gray-400 flex items-center gap-1">
              <Sparkles className="w-4 h-4 text-green-400" /> Presets:
            </span>
            <button
              type="button"
              onClick={handleAutoRouteCall}
              disabled={routingCall}
              className={`px-4 py-2 text-xs font-bold bg-amber-500/20 hover:bg-amber-500/35 text-amber-300 border border-amber-500/40 hover:border-amber-500/60 rounded-xl transition-all duration-300 flex items-center gap-2 cursor-pointer shadow-lg shadow-amber-500/10 hover:shadow-amber-500/20 active:scale-95 disabled:opacity-50 disabled:pointer-events-none ${routingCall ? 'animate-pulse' : ''}`}
            >
              <Zap className={`w-4 h-4 text-amber-400 ${routingCall ? 'animate-spin' : ''}`} />
              {routingCall ? 'Routing call...' : '⚡ 1-Click Auto-Route Call'}
            </button>
            <button
              type="button"
              onClick={handleAutoConfigureSingleCable}
              className="px-3 py-1.5 text-xs font-semibold bg-blue-500/10 hover:bg-blue-500/20 text-blue-300 border border-blue-500/20 hover:border-blue-500/40 rounded-lg transition-all duration-300 cursor-pointer"
            >
              🔌 Auto-Detect Single Cable
            </button>
            <button
              type="button"
              onClick={handleAutoConfigureDualCable}
              className="px-3 py-1.5 text-xs font-semibold bg-purple-500/10 hover:bg-purple-500/20 text-purple-300 border border-purple-500/20 hover:border-purple-500/40 rounded-lg transition-all duration-300 cursor-pointer"
            >
              🎛️ Auto-Detect Dual Cables (A+B)
            </button>
            <button
              type="button"
              onClick={handleAutoConfigureVoicemeeter}
              className="px-3 py-1.5 text-xs font-semibold bg-green-500/10 hover:bg-green-500/20 text-green-300 border border-green-500/20 hover:border-green-500/40 rounded-lg transition-all duration-300 cursor-pointer"
            >
              🎛️ Auto-Detect Voicemeeter
            </button>
            <button
              type="button"
              onClick={() => {
                setSettings({ ...settings, audio_input_device: '', audio_output_device: '' });
                setRoutingMode('standard');
                alert("Reset audio devices to System Defaults.");
              }}
              className="px-3 py-1.5 text-xs font-semibold bg-white/5 hover:bg-white/10 text-gray-300 border border-white/10 rounded-lg transition-all duration-300 ml-auto cursor-pointer"
            >
              🔄 Reset to Defaults
            </button>
          </div>

          {/* Dropdown controls */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 bg-black/25 p-5 rounded-xl border border-white/5">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1 flex items-center gap-2">
                <Mic className="w-4 h-4 text-green-400" /> Input Device (Capture Microphone)
              </label>
              <select
                className="input-field appearance-none w-full bg-black/40"
                value={settings.audio_input_device}
                onChange={e => handleChange('audio_input_device', e.target.value)}
              >
                <option value="">🖥️ System Default Microphone</option>
                {audioDevices.filter(d => d.kind === 'input').map(d => (
                  <option key={d.name} value={d.name}>
                    {d.name.toLowerCase().includes('cable') ? '🔌 ' : '🎤 '}{d.name}
                  </option>
                ))}
              </select>
              <p className="text-[11px] text-gray-500 mt-1.5">
                {routingMode === 'standard' && 'Captures your physical microphone voice.'}
                {routingMode === 'single_cable' && <span>For Phone Link calls, select <strong className="text-green-400">CABLE Output</strong> (routes caller sound to SAATHI).</span>}
                {routingMode === 'dual_cable' && <span>Select <strong className="text-purple-400">CABLE-A Output</strong> (routes caller voice to SAATHI).</span>}
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1 flex items-center gap-2">
                <Volume2 className="w-4 h-4 text-green-400" /> Output Device (AI Voice Output)
              </label>
              <select
                className="input-field appearance-none w-full bg-black/40"
                value={settings.audio_output_device}
                onChange={e => handleChange('audio_output_device', e.target.value)}
              >
                <option value="">🖥️ System Default Speakers</option>
                {audioDevices.filter(d => d.kind === 'output').map(d => (
                  <option key={d.name} value={d.name}>
                    {d.name.toLowerCase().includes('cable') ? '🔌 ' : '🔊 '}{d.name}
                  </option>
                ))}
              </select>
              <p className="text-[11px] text-gray-500 mt-1.5">
                {routingMode === 'standard' && 'Plays AI speech out loud through your speakers.'}
                {routingMode === 'single_cable' && <span>Route AI voice to caller by selecting <strong className="text-green-400">CABLE Input</strong>.</span>}
                {routingMode === 'dual_cable' && <span>Route AI voice into call mic by selecting <strong className="text-purple-400">CABLE-B Input</strong>.</span>}
              </p>
            </div>
          </div>

          {/* Interactive steps Accordion Info */}
          <div className="bg-blue-500/10 border border-blue-500/20 rounded-xl p-5 text-sm text-blue-300">
            <h4 className="font-semibold text-blue-400 flex items-center gap-1.5 mb-2">
              <Info className="w-4 h-4" /> Phone Link + VB-Cable Sound Integration Guide
            </h4>
            
            {routingMode === 'standard' ? (
              <div className="text-gray-400 space-y-1">
                <p>Standard mode is perfect for pairing tests and standard usage. The AI will speak directly out of your PC speakers and listen using your PC microphone.</p>
                <p className="text-xs mt-2 text-yellow-400/80">⚠️ Note: Set this mode to verify your Sarvam TTS key out-loud first.</p>
              </div>
            ) : routingMode === 'single_cable' ? (
              <ol className="space-y-1.5 list-decimal list-inside text-gray-400 text-xs">
                <li>Download & install VB-Cable driver from <span className="text-blue-400 underline cursor-pointer">vb-audio.com</span>, then restart your PC.</li>
                <li>Set SAATHI Output to <strong className="text-white">CABLE Input</strong> and Input to <strong className="text-white">CABLE Output</strong> (or click "Auto-Detect Single Cable" preset).</li>
                <li>In **Windows Settings &rarr; Sound**, set the playback/output device for your **Phone Link** application specifically to <strong className="text-white">CABLE Input</strong>.</li>
                <li>Set the input/microphone device in the **Phone Link** app (or Windows sound input default) to <strong className="text-white">CABLE Output</strong>.</li>
                <li>SAATHI AI will automatically hear the caller and play its generated voice back to the caller natively!</li>
              </ol>
            ) : (
              <ol className="space-y-1.5 list-decimal list-inside text-gray-400 text-xs">
                <li>For fully automated background calls (where you do not hear it on speakers), install two Virtual Cables: **VB-Cable A** and **VB-Cable B**.</li>
                <li>Set SAATHI Input to <strong className="text-white">CABLE-A Output</strong>, and Output to <strong className="text-white">CABLE-B Input</strong> (or click "Auto-Detect Dual Cables" preset).</li>
                <li>In **Windows Settings &rarr; Sound**, configure **Phone Link** output to go to <strong className="text-white">CABLE-A Input</strong>, and **Phone Link** mic/input to capture from <strong className="text-white">CABLE-B Output</strong>.</li>
                <li>Calls will run silently in the background, handled end-to-end by the SAATHI voice agent while you watch the live transcript on the dashboard.</li>
              </ol>
            )}
          </div>
        </div>

        {/* Telegram Integration */}
        <div className="glass-panel p-6 space-y-6">
          <h3 className="font-semibold text-lg flex items-center gap-2 text-[#0088cc] border-b border-white/10 pb-3">
            <Shield className="w-5 h-5" /> Telegram Bot Integration
          </h3>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1">Bot Token</label>
              <input 
                type="password" 
                className="input-field"
                value={settings.telegram_bot_token}
                onChange={e => handleChange('telegram_bot_token', e.target.value)}
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-400 mb-1">Allowed Chat ID</label>
              <input 
                type="text" 
                className="input-field"
                value={settings.telegram_chat_id}
                onChange={e => handleChange('telegram_chat_id', e.target.value)}
              />
            </div>
          </div>
        </div>

      </div>
    </div>
  );
}
