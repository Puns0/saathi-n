import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { 
  Activity, 
  RotateCw, 
  Volume2, 
  Mic, 
  Radio, 
  Play, 
  Clock, 
  Terminal, 
  CheckCircle2, 
  AlertTriangle, 
  Zap, 
  Check, 
  Smartphone,
  Sliders,
  Sparkles
} from 'lucide-react';

interface AudioEndpoint {
  name: string;
  flow: 'input' | 'output';
  is_default: boolean;
  default_sample_rate: number;
  channels: number;
  sample_format: string;
  state: string;
}

interface RecordPlaybackResult {
  duration_sec: number;
  samples_count: number;
  max_rms: number;
  status: string;
}

interface TurnLatency {
  turn_id: number;
  stt_ms: number;
  llm_first_token_ms: number;
  tts_first_audio_ms: number;
  total_turn_ms: number;
  transcript: string;
  response: string;
  timestamp: string;
}

export default function Diagnostics() {
  const [endpoints, setEndpoints] = useState<AudioEndpoint[]>([]);
  const [loadingEndpoints, setLoadingEndpoints] = useState(false);
  const [searchFilter, setSearchFilter] = useState('');

  // Per-hop level meters
  const [levels, setLevels] = useState({ capture: 0, stt: 0, tts: 0 });

  // Test tone state
  const [selectedOutput, setSelectedOutput] = useState('');
  const [toneFrequency, setToneFrequency] = useState(1000);
  const [toneDuration, setToneDuration] = useState(1500);
  const [tonePlaying, setTonePlaying] = useState(false);
  const [toneStatus, setToneStatus] = useState<string | null>(null);

  // Record & playback test state
  const [loopbackInput, setLoopbackInput] = useState('');
  const [loopbackOutput, setLoopbackOutput] = useState('');
  const [recordingLoopback, setRecordingLoopback] = useState(false);
  const [loopbackCountdown, setLoopbackCountdown] = useState(0);
  const [loopbackResult, setLoopbackResult] = useState<RecordPlaybackResult | null>(null);
  const [loopbackError, setLoopbackError] = useState<string | null>(null);

  // Path D: Browser Mic & AI Pipeline Live Test
  const [browserRecording, setBrowserRecording] = useState(false);
  const [browserCountdown, setBrowserCountdown] = useState(0);
  const [browserStatus, setBrowserStatus] = useState<string | null>(null);
  const [browserTranscript, setBrowserTranscript] = useState<string | null>(null);
  const [browserAiResponse, setBrowserAiResponse] = useState<string | null>(null);
  const [testSpeechText, setTestSpeechText] = useState('Hello, this is an AI assistant testing audio.');
  const [playingSpeech, setPlayingSpeech] = useState(false);

  // Latency log
  const [latencyLogs, setLatencyLogs] = useState<TurnLatency[]>([]);

  // PowerShell Hardware Dump
  const [dumpOutput, setDumpOutput] = useState<string | null>(null);
  const [runningDump, setRunningDump] = useState(false);

  const fetchEndpoints = async () => {
    setLoadingEndpoints(true);
    try {
      const res = await invoke<AudioEndpoint[]>('get_audio_diagnostics');
      setEndpoints(res);

      // Default select endpoints if empty
      if (!selectedOutput) {
        const defOut = res.find(e => e.flow === 'output' && e.is_default);
        if (defOut) setSelectedOutput(defOut.name);
      }
      if (!loopbackInput) {
        const defIn = res.find(e => e.flow === 'input' && e.is_default);
        if (defIn) setLoopbackInput(defIn.name);
      }
      if (!loopbackOutput) {
        const defOut = res.find(e => e.flow === 'output' && e.is_default);
        if (defOut) setLoopbackOutput(defOut.name);
      }
    } catch (e) {
      console.error('Failed to get audio diagnostics:', e);
    } finally {
      setLoadingEndpoints(false);
    }
  };

  useEffect(() => {
    fetchEndpoints();

    // Listen for live level meters
    const unlistenLevels = listen('audio_level_meter', (event: any) => {
      const p = event.payload;
      if (p) {
        setLevels({
          capture: Math.min(100, Math.max(0, p.capture || 0)),
          stt: Math.min(100, Math.max(0, p.stt || 0)),
          tts: Math.min(100, Math.max(0, p.tts || 0)),
        });
      }
    });

    // Listen for turn latency events
    const unlistenLatency = listen('turn_latency', (event: any) => {
      const p = event.payload;
      if (p) {
        setLatencyLogs(prev => [
          {
            turn_id: prev.length + 1,
            stt_ms: p.stt_ms || 0,
            llm_first_token_ms: p.llm_first_token_ms || 0,
            tts_first_audio_ms: p.tts_first_audio_ms || 0,
            total_turn_ms: p.total_turn_ms || 0,
            transcript: p.transcript || '',
            response: p.response || '',
            timestamp: new Date().toLocaleTimeString(),
          },
          ...prev.slice(0, 49),
        ]);
      }
    });

    // Listen for STT transcript & AI response events
    const unlistenSTT = listen<any>('stt_transcript', (event: any) => {
      if (event.payload?.text) setBrowserTranscript(event.payload.text);
    });
    const unlistenLLM = listen<any>('llm_finished', (event: any) => {
      if (event.payload?.text) setBrowserAiResponse(event.payload.text);
    });

    return () => {
      unlistenLevels.then(f => f());
      unlistenLatency.then(f => f());
      unlistenSTT.then(f => f());
      unlistenLLM.then(f => f());
    };
  }, []);

  const handlePlayTone = async () => {
    setTonePlaying(true);
    setToneStatus('Generating and outputting test tone...');
    try {
      const res = await invoke<string>('play_test_tone', {
        outputDevice: selectedOutput,
        frequency: toneFrequency,
        durationMs: toneDuration,
      });
      setToneStatus(res);
    } catch (e: any) {
      setToneStatus(`Error: ${e}`);
    } finally {
      setTonePlaying(false);
    }
  };

  const handleRecordAndPlayback = async () => {
    setRecordingLoopback(true);
    setLoopbackError(null);
    setLoopbackResult(null);
    setLoopbackCountdown(5);

    const interval = setInterval(() => {
      setLoopbackCountdown(prev => {
        if (prev <= 1) {
          clearInterval(interval);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    try {
      const res = await invoke<RecordPlaybackResult>('record_and_playback_test', {
        inputDevice: loopbackInput,
        outputDevice: loopbackOutput,
        durationSecs: 5,
      });
      setLoopbackResult(res);
    } catch (e: any) {
      setLoopbackError(String(e));
    } finally {
      clearInterval(interval);
      setRecordingLoopback(false);
      setLoopbackCountdown(0);
    }
  };

  const handleTestSpeech = async () => {
    if (!testSpeechText.trim()) return;
    setPlayingSpeech(true);
    try {
      await invoke('say_text', { text: testSpeechText });
    } catch (e: any) {
      console.error(e);
    } finally {
      setPlayingSpeech(false);
    }
  };

  const handleBrowserMicTest = async () => {
    try {
      setBrowserRecording(true);
      setBrowserStatus('Requesting browser microphone access...');
      setBrowserTranscript(null);
      setBrowserAiResponse(null);

      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mediaRecorder = new MediaRecorder(stream);
      const chunks: Blob[] = [];

      mediaRecorder.ondataavailable = e => {
        if (e.data.size > 0) chunks.push(e.data);
      };

      setBrowserCountdown(4);
      setBrowserStatus('Listening... Speak now into your microphone!');

      let remaining = 4;
      const timer = setInterval(() => {
        remaining -= 1;
        setBrowserCountdown(remaining);
        if (remaining <= 0) {
          clearInterval(timer);
          mediaRecorder.stop();
        }
      }, 1000);

      mediaRecorder.onstop = async () => {
        stream.getTracks().forEach(t => t.stop());
        setBrowserStatus('Processing audio and sending to AI pipeline (STT -> LLM -> TTS)...');
        setBrowserRecording(false);

        try {
          const blob = new Blob(chunks, { type: mediaRecorder.mimeType || 'audio/webm' });
          const arrayBuffer = await blob.arrayBuffer();
          const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
          const decoded = await audioCtx.decodeAudioData(arrayBuffer);

          // Offline resample to 16 kHz mono
          const targetRate = 16000;
          const targetLength = Math.round(decoded.duration * targetRate);
          const offlineCtx = new OfflineAudioContext(1, targetLength, targetRate);
          const offlineSource = offlineCtx.createBufferSource();
          offlineSource.buffer = decoded;
          offlineSource.connect(offlineCtx.destination);
          offlineSource.start(0);
          const rendered = await offlineCtx.startRendering();

          const channelData = rendered.getChannelData(0);
          const int16Array = new Int16Array(channelData.length);
          for (let i = 0; i < channelData.length; i++) {
            const s = Math.max(-1, Math.min(1, channelData[i]));
            int16Array[i] = s < 0 ? s * 0x8000 : s * 0x7FFF;
          }

          await invoke('process_browser_audio', { samples: Array.from(int16Array) });
          setBrowserStatus('Audio delivered to AI Pipeline! Check transcript and audio response below.');
        } catch (err: any) {
          setBrowserStatus(`Error processing browser audio: ${err.message || err}`);
        }
      };

      mediaRecorder.start();
    } catch (err: any) {
      setBrowserRecording(false);
      setBrowserStatus(`Microphone access error: ${err.message || err}`);
    }
  };

  const handleRunDump = async () => {
    setRunningDump(true);
    setDumpOutput('Running PowerShell Bluetooth & Audio Diagnostics...');
    try {
      const res = await invoke<string>('run_bt_diagnostics_dump');
      setDumpOutput(res);
    } catch (e: any) {
      setDumpOutput(`Error running diagnostics dump: ${e}`);
    } finally {
      setRunningDump(false);
    }
  };

  const inputEndpoints = endpoints.filter(e => e.flow === 'input' && e.name.toLowerCase().includes(searchFilter.toLowerCase()));
  const outputEndpoints = endpoints.filter(e => e.flow === 'output' && e.name.toLowerCase().includes(searchFilter.toLowerCase()));

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500 pb-20">
      {/* Header */}
      <header className="flex flex-col md:flex-row justify-between items-start md:items-end gap-4">
        <div>
          <div className="flex items-center gap-3 mb-1">
            <h2 className="text-3xl font-bold tracking-tight">Audio & Latency Diagnostics</h2>
            <span className="bg-primary/20 text-primary text-xs font-semibold px-2.5 py-0.5 rounded-full border border-primary/30 flex items-center gap-1">
              <Sparkles className="w-3 h-3 text-yellow-400" /> Phase 0 Tooling
            </span>
          </div>
          <p className="text-gray-400">
            Live audio endpoint discovery, per-hop VU level meters, loopback verification, and conversational latency logs.
          </p>
        </div>

        <button
          onClick={fetchEndpoints}
          disabled={loadingEndpoints}
          className="btn-primary flex items-center gap-2"
        >
          <RotateCw className={`w-4 h-4 ${loadingEndpoints ? 'animate-spin' : ''}`} />
          {loadingEndpoints ? 'Scanning...' : 'Refresh Endpoints'}
        </button>
      </header>

      {/* 1. Per-Hop Live Level Meters */}
      <div className="glass-panel p-6 space-y-5 relative overflow-hidden">
        <div className="flex items-center justify-between border-b border-white/10 pb-3">
          <h3 className="font-semibold text-lg flex items-center gap-2 text-cyan-400">
            <Sliders className="w-5 h-5 text-cyan-400" /> Per-Hop Level Meters (Live Audio Flow)
          </h3>
          <span className="text-xs text-gray-500 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            Live Reactive Feed
          </span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {/* Hop 1: Capture Device */}
          <div className="bg-black/30 p-4 rounded-xl border border-white/5 space-y-3">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-300 font-medium flex items-center gap-1.5">
                <Mic className="w-4 h-4 text-emerald-400" /> Hop 1: Capture Device
              </span>
              <span className="font-mono text-emerald-400 font-bold">{levels.capture.toFixed(1)}%</span>
            </div>
            <div className="w-full bg-black/60 h-3.5 rounded-full overflow-hidden p-0.5 border border-white/10">
              <div 
                className="h-full rounded-full transition-all duration-100 bg-gradient-to-r from-emerald-500 via-yellow-400 to-red-500"
                style={{ width: `${levels.capture}%` }}
              />
            </div>
            <p className="text-[10px] text-gray-500">Raw audio signal entering from selected input microphone.</p>
          </div>

          {/* Hop 2: STT Input */}
          <div className="bg-black/30 p-4 rounded-xl border border-white/5 space-y-3">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-300 font-medium flex items-center gap-1.5">
                <Radio className="w-4 h-4 text-indigo-400" /> Hop 2: STT Input (VAD Active)
              </span>
              <span className="font-mono text-indigo-400 font-bold">{levels.stt.toFixed(1)}%</span>
            </div>
            <div className="w-full bg-black/60 h-3.5 rounded-full overflow-hidden p-0.5 border border-white/10">
              <div 
                className="h-full rounded-full transition-all duration-100 bg-gradient-to-r from-indigo-500 via-purple-400 to-pink-500"
                style={{ width: `${levels.stt}%` }}
              />
            </div>
            <p className="text-[10px] text-gray-500">Filtered speech energy triggering VAD and routed to Sarvam/Whisper.</p>
          </div>

          {/* Hop 3: TTS Output */}
          <div className="bg-black/30 p-4 rounded-xl border border-white/5 space-y-3">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-300 font-medium flex items-center gap-1.5">
                <Volume2 className="w-4 h-4 text-amber-400" /> Hop 3: TTS Output (AI Spoken Voice)
              </span>
              <span className="font-mono text-amber-400 font-bold">{levels.tts.toFixed(1)}%</span>
            </div>
            <div className="w-full bg-black/60 h-3.5 rounded-full overflow-hidden p-0.5 border border-white/10">
              <div 
                className="h-full rounded-full transition-all duration-100 bg-gradient-to-r from-amber-500 via-orange-400 to-red-500"
                style={{ width: `${levels.tts}%` }}
              />
            </div>
            <p className="text-[10px] text-gray-500">Synthesized AI voice energy sent to speaker/Voicemeeter.</p>
          </div>
        </div>
      </div>

      {/* 2. Audio Verification & Loopback Tools */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Tool A: Play 1 kHz Test Tone */}
        <div className="glass-panel p-6 space-y-5">
          <div className="border-b border-white/10 pb-3">
            <h3 className="font-semibold text-lg flex items-center gap-2 text-yellow-400">
              <Volume2 className="w-5 h-5 text-yellow-400" /> Play 1 kHz Test Tone
            </h3>
            <p className="text-xs text-gray-400 mt-1">
              Verify that an output device (PC speakers, Voicemeeter, or phone headset) receives audible sound.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-gray-400 mb-1">Target Output Device</label>
              <select
                className="input-field w-full text-xs bg-black/40"
                value={selectedOutput}
                onChange={e => setSelectedOutput(e.target.value)}
              >
                <option value="">🖥️ Default Output Device</option>
                {outputEndpoints.map(d => (
                  <option key={d.name} value={d.name}>
                    {d.is_default ? '★ [Default] ' : ''}{d.name} ({d.default_sample_rate}Hz)
                  </option>
                ))}
              </select>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-medium text-gray-400 mb-1">Frequency (Hz)</label>
                <input
                  type="number"
                  className="input-field w-full text-xs font-mono"
                  value={toneFrequency}
                  onChange={e => setToneFrequency(Number(e.target.value))}
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-400 mb-1">Duration (ms)</label>
                <input
                  type="number"
                  className="input-field w-full text-xs font-mono"
                  value={toneDuration}
                  onChange={e => setToneDuration(Number(e.target.value))}
                />
              </div>
            </div>

            <button
              onClick={handlePlayTone}
              disabled={tonePlaying}
              className="btn-primary w-full flex items-center justify-center gap-2 bg-yellow-600 hover:bg-yellow-500 text-black font-semibold"
            >
              <Play className={`w-4 h-4 ${tonePlaying ? 'animate-spin' : ''}`} />
              {tonePlaying ? 'Playing Tone...' : 'Play 1 kHz Test Tone'}
            </button>

            {toneStatus && (
              <div className="p-3 bg-white/5 rounded-lg border border-white/10 text-xs text-gray-300 font-mono">
                {toneStatus}
              </div>
            )}
          </div>
        </div>

        {/* Tool B: Record 5s & Play Back */}
        <div className="glass-panel p-6 space-y-5">
          <div className="border-b border-white/10 pb-3">
            <h3 className="font-semibold text-lg flex items-center gap-2 text-emerald-400">
              <Mic className="w-5 h-5 text-emerald-400" /> Record 5s & Play Back (Loopback)
            </h3>
            <p className="text-xs text-gray-400 mt-1">
              Captures 5 seconds from selected input and plays it back to output to prove bidirectional audio.
            </p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-gray-400 mb-1">Capture Input Device (Microphone)</label>
              <select
                className="input-field w-full text-xs bg-black/40"
                value={loopbackInput}
                onChange={e => setLoopbackInput(e.target.value)}
              >
                <option value="">🎤 Default Input Device</option>
                {inputEndpoints.map(d => (
                  <option key={d.name} value={d.name}>
                    {d.is_default ? '★ [Default] ' : ''}{d.name} ({d.default_sample_rate}Hz)
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs font-medium text-gray-400 mb-1">Playback Output Device</label>
              <select
                className="input-field w-full text-xs bg-black/40"
                value={loopbackOutput}
                onChange={e => setLoopbackOutput(e.target.value)}
              >
                <option value="">🔊 Default Output Device</option>
                {outputEndpoints.map(d => (
                  <option key={d.name} value={d.name}>
                    {d.is_default ? '★ [Default] ' : ''}{d.name}
                  </option>
                ))}
              </select>
            </div>

            <button
              onClick={handleRecordAndPlayback}
              disabled={recordingLoopback}
              className="btn-primary w-full flex items-center justify-center gap-2 bg-emerald-600 hover:bg-emerald-500 text-black font-semibold"
            >
              <Radio className={`w-4 h-4 ${recordingLoopback ? 'animate-pulse text-red-950' : ''}`} />
              {recordingLoopback ? `Recording... (${loopbackCountdown}s remaining)` : 'Start 5s Loopback Test'}
            </button>

            {loopbackResult && (
              <div className="p-3 bg-emerald-500/10 rounded-lg border border-emerald-500/20 text-xs text-emerald-300 space-y-1">
                <div className="font-semibold flex items-center gap-1.5">
                  <CheckCircle2 className="w-4 h-4 text-emerald-400" /> Loopback Test Complete
                </div>
                <div className="font-mono text-[11px] text-gray-400">
                  Samples: {loopbackResult.samples_count.toLocaleString()} | Peak RMS: {loopbackResult.max_rms.toFixed(4)}
                </div>
                <div className="text-gray-300 text-[11px]">{loopbackResult.status}</div>
              </div>
            )}

            {loopbackError && (
              <div className="p-3 bg-red-500/10 rounded-lg border border-red-500/20 text-xs text-red-300 flex items-center gap-1.5">
                <AlertTriangle className="w-4 h-4 text-red-400 shrink-0" />
                {loopbackError}
              </div>
            )}
          </div>
        </div>

        {/* Tool C: Path D Browser Mic & Live Pipeline Test */}
        <div className="glass-panel p-6 space-y-5 lg:col-span-2 border border-cyan-500/30">
          <div className="border-b border-white/10 pb-3 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-2">
            <div>
              <h3 className="font-semibold text-lg flex items-center gap-2 text-cyan-400">
                <Mic className="w-5 h-5 text-cyan-400" /> Path D: Browser Mic & AI Pipeline Live Test
              </h3>
              <p className="text-xs text-gray-400 mt-1">
                Records directly from browser microphone, converts to 16 kHz PCM, and routes through STT &rarr; Groq LLM &rarr; TTS speaker output.
              </p>
            </div>
            <span className="bg-cyan-500/20 text-cyan-300 text-[10px] font-bold px-2 py-0.5 rounded border border-cyan-500/30 shrink-0">
              PATH D FALLBACK
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Left: Mic Record */}
            <div className="space-y-3">
              <button
                onClick={handleBrowserMicTest}
                disabled={browserRecording}
                className="btn-primary w-full flex items-center justify-center gap-2 bg-cyan-600 hover:bg-cyan-500 text-black font-semibold h-11"
              >
                <Radio className={`w-4 h-4 ${browserRecording ? 'animate-pulse text-red-950' : ''}`} />
                {browserRecording ? `Recording Voice... (${browserCountdown}s remaining)` : 'Record Mic (4s) & Run Pipeline'}
              </button>

              {browserStatus && (
                <div className="p-3 bg-cyan-500/10 rounded-lg border border-cyan-500/20 text-xs text-cyan-300">
                  {browserStatus}
                </div>
              )}

              {browserTranscript && (
                <div className="p-3 bg-black/40 rounded-lg border border-white/10 text-xs space-y-1">
                  <span className="text-gray-400 font-semibold block text-[10px] uppercase">User Transcript (STT):</span>
                  <div className="text-white font-medium">{browserTranscript}</div>
                </div>
              )}

              {browserAiResponse && (
                <div className="p-3 bg-purple-500/10 rounded-lg border border-purple-500/20 text-xs space-y-1">
                  <span className="text-purple-300 font-semibold block text-[10px] uppercase">AI Response (Spoken):</span>
                  <div className="text-purple-100">{browserAiResponse}</div>
                </div>
              )}
            </div>

            {/* Right: TTS Spoken Voice Direct Test */}
            <div className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-gray-400 mb-1">Test AI Spoken Response (TTS)</label>
                <input
                  type="text"
                  className="input-field w-full text-xs bg-black/40"
                  value={testSpeechText}
                  onChange={e => setTestSpeechText(e.target.value)}
                  placeholder="Enter text to speak..."
                />
              </div>

              <button
                onClick={handleTestSpeech}
                disabled={playingSpeech}
                className="btn-secondary w-full flex items-center justify-center gap-2 border-white/20 text-white font-medium h-11"
              >
                <Volume2 className="w-4 h-4 text-amber-400" />
                {playingSpeech ? 'Speaking...' : 'Synthesize & Speak Text'}
              </button>
              <p className="text-[11px] text-gray-500">
                Immediately plays speech through the currently configured audio output device to verify speaker routing.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* 3. Per-Turn Latency Monitor */}
      <div className="glass-panel p-6 space-y-4">
        <div className="flex items-center justify-between border-b border-white/10 pb-3">
          <div>
            <h3 className="font-semibold text-lg flex items-center gap-2 text-purple-400">
              <Clock className="w-5 h-5 text-purple-400" /> Conversational Turn Latency Monitor
            </h3>
            <p className="text-xs text-gray-400 mt-1">
              Tracks VAD Speech End &rarr; STT Transcript &rarr; LLM First Token &rarr; TTS First Audio (Target: &le; 2.0s).
            </p>
          </div>
          <span className="text-xs font-mono text-gray-500 bg-white/5 px-2.5 py-1 rounded-md">
            {latencyLogs.length} turns recorded
          </span>
        </div>

        {latencyLogs.length === 0 ? (
          <div className="p-8 text-center text-gray-500 text-xs bg-black/20 rounded-xl border border-white/5">
            No live conversation turns yet. Make a call or test speech input to see turn latency breakdowns in real-time.
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-white/10 text-gray-400 font-medium">
                  <th className="py-2.5 px-3">#</th>
                  <th className="py-2.5 px-3">Time</th>
                  <th className="py-2.5 px-3">STT Time</th>
                  <th className="py-2.5 px-3">LLM 1st Token</th>
                  <th className="py-2.5 px-3">TTS 1st Audio</th>
                  <th className="py-2.5 px-3">Total Turn</th>
                  <th className="py-2.5 px-3">User Spoken</th>
                  <th className="py-2.5 px-3">AI Reply</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-white/5 font-mono">
                {latencyLogs.map((log) => {
                  const isFast = log.total_turn_ms <= 2000;
                  const isModerate = log.total_turn_ms > 2000 && log.total_turn_ms <= 3000;
                  return (
                    <tr key={log.turn_id} className="hover:bg-white/5 transition-colors">
                      <td className="py-2.5 px-3 text-gray-400">{log.turn_id}</td>
                      <td className="py-2.5 px-3 text-gray-500">{log.timestamp}</td>
                      <td className="py-2.5 px-3 text-cyan-300">{log.stt_ms}ms</td>
                      <td className="py-2.5 px-3 text-indigo-300">{log.llm_first_token_ms}ms</td>
                      <td className="py-2.5 px-3 text-amber-300">{log.tts_first_audio_ms}ms</td>
                      <td className="py-2.5 px-3 font-bold">
                        <span className={`px-2 py-0.5 rounded text-[11px] ${
                          isFast 
                            ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/30' 
                            : isModerate 
                            ? 'bg-yellow-500/20 text-yellow-300 border border-yellow-500/30' 
                            : 'bg-red-500/20 text-red-300 border border-red-500/30'
                        }`}>
                          {(log.total_turn_ms / 1000).toFixed(2)}s
                        </span>
                      </td>
                      <td className="py-2.5 px-3 font-sans text-gray-300 max-w-xs truncate" title={log.transcript}>
                        {log.transcript}
                      </td>
                      <td className="py-2.5 px-3 font-sans text-gray-400 max-w-xs truncate" title={log.response}>
                        {log.response}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* 4. Windows Audio Endpoints Registry & Live List */}
      <div className="glass-panel p-6 space-y-4">
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 border-b border-white/10 pb-3">
          <div>
            <h3 className="font-semibold text-lg flex items-center gap-2 text-white">
              <Activity className="w-5 h-5 text-primary" /> Windows Audio Endpoints (WASAPI/CPAL)
            </h3>
            <p className="text-xs text-gray-400 mt-1">
              Active system audio input and output endpoints refreshable at any point during live call.
            </p>
          </div>
          <input
            type="text"
            placeholder="Filter devices..."
            className="input-field text-xs w-48 bg-black/40"
            value={searchFilter}
            onChange={e => setSearchFilter(e.target.value)}
          />
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Inputs */}
          <div className="space-y-3">
            <h4 className="text-xs font-semibold text-emerald-400 uppercase tracking-wider flex items-center gap-1.5">
              <Mic className="w-4 h-4" /> Capture Endpoints ({inputEndpoints.length})
            </h4>
            <div className="space-y-2 max-h-72 overflow-y-auto pr-1">
              {inputEndpoints.map(d => (
                <div key={d.name} className="p-3 bg-black/30 rounded-xl border border-white/5 flex items-center justify-between text-xs hover:border-white/20 transition-colors">
                  <div>
                    <div className="font-medium text-gray-200 flex items-center gap-2">
                      {d.name}
                      {d.is_default && (
                        <span className="bg-emerald-500/20 text-emerald-400 text-[10px] px-1.5 py-0.5 rounded font-bold border border-emerald-500/30">
                          DEFAULT
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-gray-500 font-mono mt-0.5">
                      {d.default_sample_rate} Hz • {d.channels} ch • {d.sample_format}
                    </div>
                  </div>
                  <span className="text-[10px] bg-white/5 text-gray-400 px-2 py-1 rounded">
                    {d.state}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Outputs */}
          <div className="space-y-3">
            <h4 className="text-xs font-semibold text-yellow-400 uppercase tracking-wider flex items-center gap-1.5">
              <Volume2 className="w-4 h-4" /> Render Endpoints ({outputEndpoints.length})
            </h4>
            <div className="space-y-2 max-h-72 overflow-y-auto pr-1">
              {outputEndpoints.map(d => (
                <div key={d.name} className="p-3 bg-black/30 rounded-xl border border-white/5 flex items-center justify-between text-xs hover:border-white/20 transition-colors">
                  <div>
                    <div className="font-medium text-gray-200 flex items-center gap-2">
                      {d.name}
                      {d.is_default && (
                        <span className="bg-yellow-500/20 text-yellow-400 text-[10px] px-1.5 py-0.5 rounded font-bold border border-yellow-500/30">
                          DEFAULT
                        </span>
                      )}
                    </div>
                    <div className="text-[11px] text-gray-500 font-mono mt-0.5">
                      {d.default_sample_rate} Hz • {d.channels} ch • {d.sample_format}
                    </div>
                  </div>
                  <span className="text-[10px] bg-white/5 text-gray-400 px-2 py-1 rounded">
                    {d.state}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* 5. System Bluetooth & Telephony Dump */}
      <div className="glass-panel p-6 space-y-4">
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 border-b border-white/10 pb-3">
          <div>
            <h3 className="font-semibold text-lg flex items-center gap-2 text-indigo-400">
              <Terminal className="w-5 h-5 text-indigo-400" /> Bluetooth & Telephony Hardware Dump
            </h3>
            <p className="text-xs text-gray-400 mt-1">
              Runs script <code className="text-indigo-300">dump_audio_bt.ps1</code> to inspect Windows PnP, Hands-Free Telephony profiles, and Voicemeeter status.
            </p>
          </div>
          <button
            onClick={handleRunDump}
            disabled={runningDump}
            className="btn-primary text-xs flex items-center gap-2 bg-indigo-600 hover:bg-indigo-500 text-white"
          >
            <Zap className={`w-3.5 h-3.5 ${runningDump ? 'animate-spin' : ''}`} />
            {runningDump ? 'Executing Dump...' : 'Run Hardware Dump'}
          </button>
        </div>

        {dumpOutput ? (
          <div className="bg-black/60 p-4 rounded-xl border border-white/10 font-mono text-xs text-gray-300 max-h-96 overflow-y-auto whitespace-pre leading-relaxed">
            {dumpOutput}
          </div>
        ) : (
          <div className="p-4 bg-white/5 rounded-xl border border-white/5 text-xs text-gray-500">
            Click "Run Hardware Dump" to inspect Bluetooth Hands-Free Gateway and Windows registry MMDevice properties.
          </div>
        )}
      </div>
    </div>
  );
}
