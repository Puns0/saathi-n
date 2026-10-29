import Foundation
import Combine

class WebSocketManager: ObservableObject {
    static let shared = WebSocketManager()
    
    @Published var isConnected = false
    private var webSocketTask: URLSessionWebSocketTask?
    private let callManager = CallManager.shared
    
    private init() {}
    
    func connect(to ip: String) {
        guard let url = URL(string: "ws://\(ip):7331") else { return }
        
        let session = URLSession(configuration: .default)
        webSocketTask = session.webSocketTask(with: url)
        webSocketTask?.resume()
        
        receiveMessage()
        
        // Send Hello
        let hello: [String: Any] = [
            "type": "hello",
            "client": "ios",
            "version": "1.0.0",
            "device_name": "iPhone"
        ]
        send(json: hello)
    }
    
    private func receiveMessage() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .failure(let error):
                print("WebSocket error: \(error)")
                DispatchQueue.main.async { self?.isConnected = false }
            case .success(let message):
                DispatchQueue.main.async { self?.isConnected = true }
                switch message {
                case .string(let text):
                    self?.handleMessage(text)
                case .data(let data):
                    print("Received binary data: \(data)")
                @unknown default:
                    break
                }
                self?.receiveMessage()
            }
        }
    }
    
    private func handleMessage(_ text: String) {
        guard let data = text.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let type = json["type"] as? String else { return }
              
        switch type {
        case "command_call":
            if let number = json["number"] as? String, let callId = json["call_id"] as? String {
                callManager.initiateCall(to: number, callId: callId)
            }
        case "command_hangup":
            if let callId = json["call_id"] as? String {
                callManager.endCall(callId: callId)
            }
        case "heartbeat":
            send(json: ["type": "heartbeat_ack"])
        default:
            break
        }
    }
    
    func send(json: [String: Any]) {
        guard let data = try? JSONSerialization.data(withJSONObject: json),
              let text = String(data: data, encoding: .utf8) else { return }
              
        let message = URLSessionWebSocketTask.Message.string(text)
        webSocketTask?.send(message) { error in
            if let error = error {
                print("WebSocket send error: \(error)")
            }
        }
    }
}
