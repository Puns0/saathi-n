# VoidCaller (iOS App)

VoidCaller is the iOS component of the VoidStack ecosystem. It connects to the VoidRelay PC backend over WebSocket, allowing the AI to dial numbers and route audio back to the PC via Bluetooth Hands-Free Profile (HFP).

## Architecture Overview

- **UI:** SwiftUI.
- **Networking:** `URLSessionWebSocketTask` for communicating with VoidRelay.
- **Telephony:** `CallKit` and standard `tel://` URLs for initiating calls.
- **Audio Routing:** `AVAudioSession` with `.allowBluetooth` and `.allowBluetoothA2DP` options to force audio routing to the PC.

## Prerequisites

- macOS with Xcode installed.
- An iOS device running iOS 14.0 or higher.
- An active Apple Developer Account (required to provision apps with CallKit and Background Modes on a physical device).

## Building and Running

1. Open the `VoidCaller.xcodeproj` file in Xcode.
2. Select your development team in the project target settings under "Signing & Capabilities".
3. Connect your iPhone to your Mac.
4. Select your iPhone as the run destination.
5. Click **Run** (`Cmd + R`) to install and launch the app.

## Setup Instructions

1. Open VoidCaller on your iPhone.
2. The app will prompt for necessary permissions:
   - Microphone
   - Bluetooth
   - Local Network
3. Grant all requested permissions.
4. Tap the **Settings** (gear icon) in the top right.
5. Enter the IP Address of your PC running VoidRelay (as shown in the VoidRelay setup wizard).
6. Return to the main screen and tap **Start Relay Connection**.
7. The status should change to "Connected to PC", and the large indicator should turn ON.

## Limitations on iOS

Due to strict Apple sandboxing and private API restrictions, a standard iOS app cannot:
1. Programmatically answer an incoming cellular call.
2. Programmatically end an active cellular call (unless it was initiated via a VoIP push managed by CallKit).
3. Access the raw audio stream of a cellular phone call.

To work around these limitations, VoidCaller relies on the Bluetooth Hands-Free Profile (HFP). The PC acts as a Bluetooth headset, allowing it to inject audio (AI speech) and record audio (user speech) through standard Bluetooth protocols.
