import SwiftUI

@main
struct VoidCallerIOSApp: App {
    @StateObject private var wsManager = WebSocketManager.shared
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(wsManager)
                .onAppear {
                    // Start background task & WS connection
                    wsManager.connect(to: "192.168.1.100") // Stub IP
                }
        }
    }
}
