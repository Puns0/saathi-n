import SwiftUI

struct ContentView: View {
    @EnvironmentObject var wsManager: WebSocketManager
    
    var body: some View {
        ZStack {
            Color.black.ignoresSafeArea()
            
            VStack(spacing: 24) {
                Text("VoidCaller")
                    .font(.largeTitle)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                
                Text("Dumb Pipe Mode Active")
                    .font(.headline)
                    .foregroundColor(.gray)
                
                Spacer().frame(height: 40)
                
                VStack(spacing: 16) {
                    HStack {
                        Text("Connection Status:")
                            .foregroundColor(.white)
                        Spacer()
                        Text(wsManager.isConnected ? "Connected" : "Disconnected")
                            .foregroundColor(wsManager.isConnected ? .green : .red)
                            .fontWeight(.bold)
                    }
                    
                    Text("Background connection to VoidRelay PC is active. This app relies on CallKit and Shortcuts for zero-intervention dialing.")
                        .font(.footnote)
                        .foregroundColor(.gray)
                        .multilineTextAlignment(.leading)
                }
                .padding()
                .background(Color.white.opacity(0.1))
                .cornerRadius(12)
                .padding(.horizontal)
            }
            .padding()
        }
    }
}
