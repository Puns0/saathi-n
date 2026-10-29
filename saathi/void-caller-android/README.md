# VoidCaller (Android App)

VoidCaller is the Android component of the VoidStack ecosystem. It connects to the VoidRelay PC backend over WebSocket, allowing the AI to dial numbers, answer calls, and route audio back to the PC via Bluetooth Hands-Free Profile (HFP).

## Architecture Overview

- **UI:** Jetpack Compose, Material Design 3.
- **Networking:** OkHttp WebSocket client.
- **Telephony:** Android Telecom API for Call Control (`CallManager`, `VoidConnectionService`).
- **Audio Routing:** `AudioManager` overrides to force call audio to the connected Bluetooth HFP device.
- **Background Execution:** `ConnectionService` (Foreground Service) ensures the WebSocket remains alive while the app is in the background.

## Prerequisites

- Android Studio installed.
- An Android device running Android 9.0 (API 28) or higher.

## Building and Running

1. Open the `void-caller-android` directory in Android Studio.
2. Sync the project with Gradle files.
3. Connect your Android device via USB or Wireless Debugging.
4. Click **Run** (`Shift + F10`) to install the app on your device.

## Setup Instructions

1. Open VoidCaller on your Android phone.
2. The app will prompt for necessary permissions:
   - Call Phone
   - Read Phone State
   - Answer Phone Calls
   - Manage Own Calls
   - Bluetooth Connect
   - Notifications (for the Foreground Service)
3. Grant all requested permissions.
4. Go to **Settings** (gear icon) in VoidCaller.
5. Enter the IP Address of your PC running VoidRelay (as shown in the VoidRelay setup wizard).
6. Return to the Home Screen and click **Start Relay Connection**.
7. The status should change to "Connected to PC", and the large indicator should turn ON.

## How it Works

When the app is connected to the Relay:
- **Outgoing Calls:** VoidRelay sends a WebSocket command to dial a number. VoidCaller uses `Intent.ACTION_CALL` to initiate the call.
- **Incoming Calls:** VoidCaller's `CallStateReceiver` detects an incoming call and notifies VoidRelay over WebSocket.
- **Call Control:** VoidRelay can send commands to answer or hang up calls.
- **Audio Routing:** During an active call, VoidCaller forces the audio to route through the connected Bluetooth HFP device (which is the PC running VoidRelay).
