import Foundation
import AVFoundation

class AudioRouter {
    static let shared = AudioRouter()
    
    private let audioSession = AVAudioSession.sharedInstance()
    
    func routeToBluetoothHFP() {
        do {
            try audioSession.setCategory(.playAndRecord, mode: .voiceChat, options: [.allowBluetoothHFP, .allowBluetoothA2DP])
            try audioSession.setActive(true)
            
            // Wait a moment for the session to become active
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                self.forceBluetoothRoute()
            }
        } catch {
            print("Failed to set audio session category: \(error)")
        }
    }
    
    private func forceBluetoothRoute() {
        let currentRoute = audioSession.currentRoute
        let isBluetooth = currentRoute.outputs.contains {
            $0.portType == .bluetoothHFP || $0.portType == .bluetoothA2DP || $0.portType == .bluetoothLE
        }
        
        if !isBluetooth {
            let availableInputs = audioSession.availableInputs ?? []
            if let bluetoothInput = availableInputs.first(where: {
                $0.portType == .bluetoothHFP
            }) {
                do {
                    try audioSession.setPreferredInput(bluetoothInput)
                    print("Successfully forced Bluetooth route")
                } catch {
                    print("Failed to force Bluetooth route: \(error)")
                }
            } else {
                print("No Bluetooth HFP input available")
            }
        }
    }
}
