import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../stores/configStore';
import { useBluetoothStore } from '../stores/bluetoothStore';
import { CheckCircle2, ChevronRight, Copy, RefreshCw, Smartphone, Key, Bot, ShieldAlert } from 'lucide-react';

export default function Setup() {
  const [step, setStep] = useState(1);
  const { config, updateConfig } = useConfigStore();
  const { devices, isScanning, setDevices, setScanning } = useBluetoothStore();
  
  const [ips, setIps] = useState<string[]>([]);
  const [testingGroq, setTestingGroq] = useState(false);
  const [testingSarvam, setTestingSarvam] = useState(false);
  const [testingTelegram, setTestingTelegram] = useState(false);

  useEffect(() => {
    invoke<string[]>('get_network_ips').then(setIps);
  }, []);

  const handleScanBt = async () => {
    setScanning(true);
    try {
      const devs = await invoke<any[]>('scan_bluetooth');
      setDevices(devs);
    } catch (e) {
      console.error(e);
    }
    setScanning(false);
  };

  const nextStep = () => setStep(s => Math.min(s + 1, 6));
  const prevStep = () => setStep(s => Math.max(s - 1, 1));

  return (
    <div className="max-w-3xl mx-auto py-12 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="mb-8 flex justify-between items-center">
        <h2 className="text-3xl font-bold">System Setup</h2>
        <div className="text-sm text-gray-500">Step {step} of 6</div>
      </div>

      <div className="flex gap-2 mb-8">
        {[1, 2, 3, 4, 5, 6].map(i => (
          <div key={i} className={`h-2 flex-1 rounded-full ${i <= step ? 'bg-primary' : 'bg-white/10'}`} />
        ))}
      </div>

      <div className="glass-panel p-8 min-h-[400px] flex flex-col">
        {step === 1 && (
          <div className="flex-1 space-y-6">
            <h3 className="text-xl font-semibold flex items-center gap-2"><Smartphone className="w-5 h-5 text-primary" /> Network Identity</h3>
            <p className="text-gray-400">VoidRelay needs to be accessible to your phone. Ensure both are on the same WiFi network.</p>
            <div className="space-y-3">
              {ips.map(ip => (
                <div key={ip} className="flex justify-between items-center p-4 bg-black/20 border border-white/5 rounded-lg">
                  <span className="font-mono text-lg">{ip}</span>
                  <button onClick={() => navigator.clipboard.writeText(ip)} className="p-2 hover:bg-white/10 rounded transition-colors" title="Copy IP">
                    <Copy className="w-4 h-4 text-gray-400" />
                  </button>
                </div>
              ))}
            </div>
            <p className="text-sm text-amber-400/80 bg-amber-400/10 p-3 rounded-lg flex gap-2">
              <ShieldAlert className="w-4 h-4 shrink-0" />
              Keep port 7331 open in your firewall.
            </p>
          </div>
        )}

        {step === 2 && (
          <div className="flex-1 space-y-6">
            <h3 className="text-xl font-semibold flex items-center gap-2"><RefreshCw className="w-5 h-5 text-secondary" /> Bluetooth HFP Setup</h3>
            <p className="text-gray-400">VoidRelay routes call audio over Bluetooth. Pair your phone to your PC now.</p>
            <button onClick={handleScanBt} disabled={isScanning} className="btn-secondary flex items-center gap-2">
              <RefreshCw className={`w-4 h-4 ${isScanning ? 'animate-spin' : ''}`} /> Scan for Paired Devices
            </button>
            <div className="space-y-2">
              {devices.map(d => (
                <div key={d.id} className="p-3 bg-black/20 border border-white/5 rounded flex justify-between items-center">
                  <span>{d.name || d.address}</span>
                  {d.connected && <span className="text-xs text-success bg-success/10 px-2 py-1 rounded">Connected</span>}
                </div>
              ))}
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="flex-1 space-y-6 flex flex-col items-center justify-center text-center">
            <h3 className="text-xl font-semibold text-primary">Connect VoidCaller App</h3>
            <p className="text-gray-400 max-w-md">Open VoidCaller on your phone, go to Settings, and enter one of the IP addresses from Step 1.</p>
            <div className="p-6 bg-white rounded-xl mt-4">
              <div className="w-48 h-48 bg-gray-200 flex items-center justify-center text-gray-500 text-sm">
                (QR Code: ws://{ips[0]}:7331)
              </div>
            </div>
          </div>
        )}

        {step === 4 && (
          <div className="flex-1 space-y-6">
            <h3 className="text-xl font-semibold flex items-center gap-2"><Key className="w-5 h-5 text-accent" /> API Configuration</h3>
            <p className="text-gray-400">Configure online APIs for high-quality routing.</p>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Groq API Key (Free LLM)</label>
                <div className="flex gap-2">
                  <input type="password" value={config.groq_api_key} onChange={e => updateConfig({ groq_api_key: e.target.value })} className="input-field flex-1" placeholder="gsk_..." />
                  <button onClick={async () => {
                    setTestingGroq(true);
                    try { await invoke('test_groq_key', { key: config.groq_api_key }); alert('Valid!'); } catch(e) { alert('Invalid key'); }
                    setTestingGroq(false);
                  }} disabled={testingGroq} className="btn-secondary">{testingGroq ? '...' : 'Test'}</button>
                </div>
              </div>
              
              <div>
                <label className="block text-sm text-gray-400 mb-1">Sarvam API Key (Indic STT/TTS)</label>
                <div className="flex gap-2">
                  <input type="password" value={config.sarvam_api_key} onChange={e => updateConfig({ sarvam_api_key: e.target.value })} className="input-field flex-1" />
                  <button onClick={async () => {
                    setTestingSarvam(true);
                    try { await invoke('test_sarvam_key', { key: config.sarvam_api_key }); alert('Valid!'); } catch(e) { alert('Invalid key'); }
                    setTestingSarvam(false);
                  }} disabled={testingSarvam} className="btn-secondary">{testingSarvam ? '...' : 'Test'}</button>
                </div>
              </div>
            </div>
          </div>
        )}

        {step === 5 && (
          <div className="flex-1 space-y-6 text-center flex flex-col justify-center items-center">
            <Bot className="w-16 h-16 text-primary mb-2 opacity-50" />
            <h3 className="text-xl font-semibold">Offline AI Models</h3>
            <p className="text-gray-400 max-w-md">VoidRelay can run 100% offline. We recommend proceeding to the Models tab after setup to download optimized local models based on your hardware.</p>
          </div>
        )}

        {step === 6 && (
          <div className="flex-1 space-y-6">
            <h3 className="text-xl font-semibold text-[#0088cc]">Telegram Bot (Optional)</h3>
            <p className="text-gray-400">Control VoidRelay and receive call summaries on the go.</p>
            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Bot Token</label>
                <input type="password" value={config.telegram_bot_token} onChange={e => updateConfig({ telegram_bot_token: e.target.value })} className="input-field w-full" />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">Your Chat ID</label>
                <div className="flex gap-2">
                  <input type="text" value={config.telegram_chat_id} onChange={e => updateConfig({ telegram_chat_id: e.target.value })} className="input-field flex-1" />
                  <button onClick={async () => {
                    setTestingTelegram(true);
                    try { await invoke('test_telegram_bot', { token: config.telegram_bot_token, chatId: config.telegram_chat_id }); alert('Sent!'); } catch(e) { alert('Failed to send message'); }
                    setTestingTelegram(false);
                  }} disabled={testingTelegram} className="btn-secondary">{testingTelegram ? '...' : 'Test Msg'}</button>
                </div>
              </div>
            </div>
          </div>
        )}

        <div className="mt-8 pt-6 border-t border-white/10 flex justify-between items-center">
          <button onClick={prevStep} disabled={step === 1} className="text-gray-400 hover:text-white disabled:opacity-30 transition-colors">Back</button>
          {step < 6 ? (
            <button onClick={nextStep} className="btn-primary flex items-center gap-2">Next <ChevronRight className="w-4 h-4" /></button>
          ) : (
            <button onClick={() => {/* Save and redirect to dashboard */}} className="btn-primary flex items-center gap-2"><CheckCircle2 className="w-4 h-4" /> Finish Setup</button>
          )}
        </div>
      </div>
    </div>
  );
}
