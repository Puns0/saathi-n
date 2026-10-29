# SAATHI Audio Setup, Call Screening & Routing Guide

This document details the audio paths, Android selective auto-answer architecture, and test procedures for routing live phone calls into the PC AI brain **without Microsoft Phone Link**.

---

## 1. Path Matrix & Hardware Summary

| Audio Path | Mechanism | Latency | Hardware Required | Reliability | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Path A: Bluetooth Hands-Free (HFP)** | Windows BTHHFPAudio endpoints via Direct WASAPI / Voicemeeter | ~80 - 150 ms | None (PC Bluetooth) | High when HFP paired | **Verified on PC** (`OnePlus Nord CE5 Hands-Free HF Audio`) |
| **Path B: Wired Audio Bridge (TRRS)** | 3.5mm TRRS splitter + USB-C DAC into PC Audio In/Out | ~10 - 30 ms | USB-C DAC + CTIA TRRS Splitter + 2 AUX cables | 100% Solid Hardware | Available |
| **Path C: Acoustic Speakerphone Bridge** | Phone Speakerphone ON + PC Mic/Speakers (Half-Duplex) | ~0 ms wire latency | None (Phone placed near PC) | Guaranteed Fallback | **Ready & Configured** (400ms Mute Tail) |
| **Path D: Browser Microphone Fallback** | Web Audio API streaming 16 kHz PCM directly to pipeline | ~50 ms | None (Built-in Webview) | 100% Working | **Implemented in Diagnostics** |

---

## 2. Path A: Bluetooth Hands-Free without Phone Link

### How It Works
Windows pairs directly with the phone using the Bluetooth Hands-Free Profile (Audio Gateway). When call audio is active, Windows creates virtual endpoints for the phone:
- **Phone Render**: `Speakers (OnePlus Nord CE5 Hands-Free HF Audio)` (`{22987dc1-13d1-4788-9780-4cd5a4938bfe}`)
- **Phone Capture**: `Microphone (OnePlus Nord CE5 Hands-Free HF Audio)` (`{1afd57f2-4323-495e-be78-04bec90daaa5}`)
- **Bluetooth Service**: `Headset Audio Gateway Service` (Status: `OK`)

### Step-by-Step Setup
1. **Phone Settings**:
   - Open Phone **Settings** &rarr; **Bluetooth** &rarr; Paired Devices.
   - Tap the settings gear next to your paired PC (`MSI-PUNS0`).
   - Ensure the toggle **"Phone calls"** (or **"Call audio"**) is **ENABLED**.
2. **VoidRelay Audio Settings**:
   - Open VoidRelay &rarr; **Settings** &rarr; **Audio & Routing Devices**.
   - Input Device: `Microphone (OnePlus Nord CE5 Hands-Free HF Audio)`
   - Output Device: `Speakers (OnePlus Nord CE5 Hands-Free HF Audio)`
   - Or select **"Bluetooth Hands-Free (Path A)"** preset.
3. **Voicemeeter Mapping (Clean Dual Loop - Optional)**:
   - **Windows Default Playback**: Set to `Voicemeeter Input` (routed to B1).
   - **VoidRelay Capture**: Select `Voicemeeter Out B1` (receives caller audio).
   - **VoidRelay Output (TTS)**: Select `Voicemeeter AUX Input` (routed to B2 ONLY).
   - **Windows Default Recording**: Set to `Voicemeeter Out B2` (sends AI voice into phone mic).
   > [!IMPORTANT]
   > Do NOT route Voicemeeter AUX Input to B1, or the AI will hear its own voice and create an echo loop!

---

## 3. Path B: Wired Audio Bridge

### When to Use
If Bluetooth HFP drops or experiences Bluetooth packet interference during heavy 2.4 GHz traffic.

### Hardware Shopping List
1. **USB-C to 3.5mm Headphone Jack Adapter** (with DAC chip, e.g., Apple USB-C to 3.5mm or OnePlus DAC).
2. **3.5mm 4-pole TRRS Headset Splitter Cable** (1 male TRRS plug &rarr; 2 female 3-pole TRS jacks: 1 pink Mic, 1 green Audio).
3. **Two standard 3.5mm Male-to-Male AUX Cables**:
   - Cable 1: Phone Headphone Out &rarr; Laptop Audio Input (Microphone).
   - Cable 2: Laptop Audio Output (Headphone) &rarr; Phone Mic In.

### Wiring & Gain Notes
- **CTIA Standard**: Ensure TRRS splitter is CTIA wired (Tip: Left, Ring 1: Right, Ring 2: Ground, Sleeve: Mic).
- **Gain / Attenuation**: Set PC Line-In volume to ~50% in Windows Sound Settings to avoid clipping high line-level signals into the mic preamp.

---

## 4. Path C: Acoustic Speakerphone Bridge (Zero-Wire Fallback)

### How It Works
1. When Android auto-answers an unknown caller, Android sets `AudioManager.mode = AudioManager.MODE_IN_CALL` and enables `isSpeakerphoneOn = true`.
2. The laptop microphone captures the caller's voice directly from the phone speaker.
3. The laptop speakers play the AI's synthesized voice directly into the phone microphone.

### Half-Duplex Protection (Echo & Barge-in Prevention)
Because acoustic feedback could cause the AI to transcribe its own voice:
- **TTS Playback Mute**: VoidRelay immediately sets `suppress_capture = true` while TTS audio is playing.
- **Dynamic Mute Tail**: Capture suppression remains active for an extra **400 ms** after audio finishes playing to allow room acoustics/reverb to clear.
- **VAD Silence End**: VAD speech completion is tuned to **700 ms** for natural phone cadence.

### Physical Placement Tip
- Place the phone flat on the desk **15 to 25 cm (6 to 10 inches)** from the laptop keyboard/microphone.
- Keep phone speaker volume at **60 - 70%**.

---

## 5. Path D: Browser Microphone & Live Pipeline Test

Located in VoidRelay &rarr; **Diagnostics** &rarr; **Path D: Browser Mic & AI Pipeline Live Test**:
1. Click **"Record Mic (4s) & Run Pipeline"**.
2. Speak into your PC mic. Audio is recorded, normalized, resampled to 16 kHz PCM, and passed directly into the AI talk loop.
3. Observe live STT transcription, Groq LLM streaming, and TTS audio playback through your speakers.
4. Use **"Synthesize & Speak Text"** to verify text-to-speech output instantly.

---

## 6. Android Selective Auto-Answer & Whitelist Configuration

In `VoidCaller` Android app &rarr; **Settings**:

### Settings Available
1. **Screen Unknown Callers** (Default: OFF):
   - When enabled, automatically answers calls from numbers not in your contacts or whitelist.
   - **Fail-safe**: If the PC is disconnected from WebSocket, auto-answer is **bypassed** to ensure the phone rings normally and no calls are dropped.
2. **Treat Saved Contacts as Known** (Default: ON):
   - Queries `ContactsContract.PhoneLookup` with E.164 normalization (+91 / 0 prefix handling).
   - If a caller is in your phonebook, the call rings normally and is never auto-screened.
3. **Auto-Answer Delay**:
   - Configurable from 0s (instant) to 6s (~2 rings). Default: 3s (~1 ring).
4. **Audio Route Mode**:
   - `bluetooth`: Routes call audio through Bluetooth SCO to Windows HFP.
   - `speakerphone`: Turns speakerphone on for Path C.
   - `wired`: Keeps headset routing for Path B Aux cables.
5. **Whitelist Manager**:
   - Add specific numbers to whitelist (e.g. `9876543210`).
   - Whitelisted numbers always ring normally and bypass AI screening.
   - Delete numbers anytime with one tap.

---

## 7. Acceptance Test Verification Matrix

| Test | Objective | Procedure | Expected Result | Result |
| :--- | :--- | :--- | :--- | :--- |
| **1. Whitelist Test** | Known contacts ring normally | Add number X to whitelist or contacts; call from X. | Phone rings normally. VoidRelay receives `incoming_call` with `{known: true}`. No auto-answer; no AI speech. | **PASS** |
| **2. Screening Test** | Unknown callers auto-screened | Call from unknown number Y with screening enabled. | Phone rings for delay (3s), auto-answers. AI pauses 1.5s, then greets: `"Hello, this is an AI assistant answering for [name]..."`. Caller speaks; AI transcribes and replies. | **PASS** |
| **3. Handoff Test** | Human can take over call | Click "Take Over Call" during screening in Dashboard. | AI stops speaking immediately (`pipeline.stop()`). State changes to `HANDOFF`. Call stays connected for human. | **PASS** |
| **4. Delay Test** | Filler phrases prevent dead air | Inject > 1.0s delay in LLM first token. | AI immediately utters filler (`"Ek minute..."` / `"One moment..."`) via TTS. Caller does not hear dead air. | **PASS** |
| **5. Audio Quality Test** | Clear bidirectional audio | Run 5s Loopback Test in Diagnostics and check VU meters. | Audio is clear without robotic artifacts. RMS meters reflect clean capture, STT, and TTS hops. | **PASS** |
