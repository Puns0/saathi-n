import Foundation
import UIKit

class CallManager {
    static let shared = CallManager()
    private let wsManager = WebSocketManager.shared
    
    private init() {}
    
    func initiateCall(to number: String, callId: String) {
        // iOS strict sandboxing means we can't dial without user prompt via openURL,
        // EXCEPT if we use a pre-installed Shortcut that the user authorized.
        // We trigger the shortcut via URL scheme:
        // shortcuts://run-shortcut?name=VoidDial&input=number
        
        let encodedNumber = number.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? number
        if let url = URL(string: "shortcuts://run-shortcut?name=VoidDial&input=\(encodedNumber)") {
            DispatchQueue.main.async {
                UIApplication.shared.open(url, options: [:]) { success in
                    if success {
                        // Force audio routing to HFP
                        AudioRouter.shared.enableHFP()
                        
                        self.wsManager.send(json: [
                            "type": "call_started",
                            "client": "ios",
                            "number": number,
                            "call_id": callId
                        ])
                    }
                }
            }
        }
    }
    
    func endCall(callId: String) {
        // Hanging up programmatically on iOS is virtually impossible without private APIs
        // unless acting as a CallKit provider for VoIP. Since we are using native cellular,
        // the user must hang up. We log it.
        print("Cannot programmatically hang up native cellular call on iOS")
        
        // We tell relay that call ended
        self.wsManager.send(json: [
            "type": "call_ended",
            "client": "ios",
            "call_id": callId,
            "duration_sec": 0
        ])
    }
}
