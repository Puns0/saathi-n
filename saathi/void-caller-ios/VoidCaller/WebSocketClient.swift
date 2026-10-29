import Foundation
import Network
import UIKit

class WebSocketClient {
    private var webSocketTask: URLSessionWebSocketTask?
    var onConnectionChange: ((Bool) -> Void)?
    
    private let callManager = CallManager.shared
    
    func connect(ip: String, port: String) {
        guard let url = URL(string: "ws://\(ip):\(port)") else { return }
        
        let session = URLSession(configuration: .default)
        webSocketTask = session.webSocketTask(with: url)
        
        webSocketTask?.resume()
        onConnectionChange?(true) // Simplification: Assume connected immediately
        
        // Send initial connection message
        let deviceName = UIDevice.current.name
        let initMsg = """
        {"type": "connected", "platform": "ios", "device_name": "\(deviceName)"}
        """
        sendMessage(initMsg)
        
        receiveMessage()
    }
    
    func disconnect() {
        webSocketTask?.cancel(with: .normalClosure, reason: nil)
        webSocketTask = nil
        onConnectionChange?(false)
    }
    
    private func sendMessage(_ string: String) {
        let message = URLSessionWebSocketTask.Message.string(string)
        webSocketTask?.send(message) { error in
            if let error = error {
                print("WebSocket send error: \(error)")
            }
        }
    }
    
    private func receiveMessage() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .failure(let error):
                print("WebSocket receive error: \(error)")
                self?.onConnectionChange?(false)
            case .success(let message):
                switch message {
                case .string(let text):
                    self?.handleCommand(text)
                case .data(let data):
                    print("Received data: \(data)")
                @unknown default:
                    break
                }
                self?.receiveMessage() // Continue listening
            }
        }
    }
    
    private func handleCommand(_ text: String) {
        // Simple JSON parsing
        guard let data = text.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data, options: []) as? [String: Any],
              let command = json["command"] as? String else {
            return
        }
        
        DispatchQueue.main.async {
            switch command {
            case "dial":
                if let number = json["number"] as? String {
                    self.callManager.initiateCall(to: number)
                }
            case "hangup":
                self.callManager.endCall()
            case "answer":
                self.callManager.answerCall()
            default:
                print("Unknown command: \(command)")
            }
        }
    }
}
