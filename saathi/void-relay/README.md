# VoidRelay (PC Brain)

VoidRelay is the intelligent backend and dashboard for the VoidStack AI Voice Ecosystem. It orchestrates AI models (STT, LLM, TTS), handles cross-platform Bluetooth audio routing, and communicates with your mobile devices via WebSocket.

## Architecture Overview

- **Frontend:** React 18, TypeScript, Vite, TailwindCSS, Zustand.
- **Backend:** Rust, Tauri 2.0.
- **AI Integration:** Sarvam API, Groq API, local `whisper-rs`, local Piper TTS, and local Ollama.
- **Audio Routing:** Native Bluetooth Hands-Free Profile (HFP) integration via `cpal` and platform-specific APIs.
- **Communication:** WebSocket server running on port `7331`.

## Prerequisites

Ensure you have the Rust toolchain and Node.js installed.

### Operating System Setup

Run the setup script for your respective OS to install dependencies and configure firewall rules.

- **Linux:** `chmod +x setup-linux.sh && ./setup-linux.sh`
- **Windows:** Run `setup-windows.ps1` as Administrator.
- **macOS:** `chmod +x setup-macos.sh && ./setup-macos.sh`

## Build Instructions

To build the application for your platform:

```bash
npm install
npm run tauri build
```

This will produce the appropriate installer (e.g., `.exe`/`.msi` for Windows, `.dmg` for macOS, `.deb`/`.AppImage` for Linux) in the `src-tauri/target/release/bundle/` directory.

## First-Run Setup Guide

1. Open VoidRelay.
2. Complete the 6-step Setup Wizard:
   - Identify your PC IP for the mobile apps.
   - Pair your phone via Bluetooth.
   - Enter your Groq and Sarvam API keys.
   - Configure the Telegram Bot.
3. Navigate to the **Models** page to download and activate offline models.

## Troubleshooting

- **Bluetooth HFP Issues:** Ensure your phone is connected for *Calls/Audio*. On Linux, verify PulseAudio/PipeWire configuration. On macOS, ensure Bluetooth permissions are granted.
- **Connection Refused:** Verify that port `7331` is allowed through your firewall.
- **Ollama Models Not Found:** Ensure the Ollama service is running (`ollama serve`) and that you have pulled the required models (e.g., `ollama pull llama3.2:3b`).
