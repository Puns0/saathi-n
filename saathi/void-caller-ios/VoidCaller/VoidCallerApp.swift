import SwiftUI

@main
struct VoidCallerApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
                // Set the default status bar style
                .preferredColorScheme(.dark)
        }
    }
}

class AppState: ObservableObject {
    @Published var isConnected = false
    @Published var serverIp: String {
        didSet {
            UserDefaults.standard.set(serverIp, forKey: "server_ip")
        }
    }
    @Published var serverPort: String {
        didSet {
            UserDefaults.standard.set(serverPort, forKey: "server_port")
        }
    }
    
    let webSocketClient = WebSocketClient()

    init() {
        self.serverIp = UserDefaults.standard.string(forKey: "server_ip") ?? "192.168.1.100"
        self.serverPort = UserDefaults.standard.string(forKey: "server_port") ?? "7331"
        
        webSocketClient.onConnectionChange = { [weak self] connected in
            DispatchQueue.main.async {
                self?.isConnected = connected
            }
        }
    }
    
    func connect() {
        webSocketClient.connect(ip: serverIp, port: serverPort)
    }
    
    func disconnect() {
        webSocketClient.disconnect()
    }
}
