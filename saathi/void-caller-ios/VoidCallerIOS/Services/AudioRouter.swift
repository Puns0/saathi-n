import Foundation
import AVFoundation

class AudioRouter {
    static let shared = AudioRouter()
    
    private init() {}
    
    func enableHFP() {
        let session = AVAudioSession.sharedInstance()
        do {
            // mode = voiceChat forces the system to look for HFP profiles
            try session.setCategory(.playAndRecord, mode: .voiceChat, options: [.allowBluetooth, .allowBluetoothA2DP])
            try session.setActive(true)
            print("Audio routed to Bluetooth HFP successfully.")
        } catch {
            print("Failed to route audio: \(error.localizedDescription)")
        }
    }
}
