import SwiftUI

struct ContentView: View {
    @EnvironmentObject var appState: AppState
    @State private var showingSettings = false

    var body: some View {
        NavigationView {
            ZStack {
                Color(red: 10/255, green: 10/255, blue: 15/255)
                    .ignoresSafeArea()

                VStack(spacing: 32) {
                    Spacer()

                    // Connection Indicator
                    ZStack {
                        Circle()
                            .fill(appState.isConnected ? Color.blue.opacity(0.2) : Color(red: 22/255, green: 22/255, blue: 30/255))
                            .frame(width: 150, height: 150)
                            .overlay(
                                Circle().stroke(appState.isConnected ? Color.blue.opacity(0.5) : Color.white.opacity(0.1), lineWidth: 2)
                            )
                        
                        Circle()
                            .fill(appState.isConnected ? Color.blue : Color(red: 22/255, green: 22/255, blue: 30/255))
                            .frame(width: 120, height: 120)
                        
                        Text(appState.isConnected ? "ON" : "OFF")
                            .font(.system(size: 32, weight: .bold))
                            .foregroundColor(.white)
                    }

                    VStack(spacing: 8) {
                        Text("Relay Connection")
                            .font(.title2)
                            .fontWeight(.bold)
                            .foregroundColor(.white)
                        
                        Text(appState.isConnected ? "Connected to PC" : "Disconnected")
                            .foregroundColor(.gray)
                    }

                    Spacer()

                    Button(action: {
                        if appState.isConnected {
                            appState.disconnect()
                        } else {
                            appState.connect()
                        }
                    }) {
                        Text(appState.isConnected ? "Stop Relay Connection" : "Start Relay Connection")
                            .font(.headline)
                            .fontWeight(.bold)
                            .foregroundColor(.white)
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(appState.isConnected ? Color.red : Color.blue)
                            .cornerRadius(12)
                    }
                    .padding(.horizontal, 24)
                    .padding(.bottom, 32)
                }
            }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .principal) {
                    Text("VoidCaller")
                        .font(.headline)
                        .foregroundColor(.white)
                }
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button(action: { showingSettings = true }) {
                        Image(systemName: "gear")
                            .foregroundColor(.gray)
                    }
                }
            }
            .sheet(isPresented: $showingSettings) {
                SettingsView()
                    .environmentObject(appState)
            }
        }
    }
}
