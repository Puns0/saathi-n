import Foundation
import CallKit
import AVFoundation
import UIKit

class CallManager: NSObject {
    static let shared = CallManager()
    
    private let callController = CXCallController()
    private var currentCallUUID: UUID?
    
    private let audioRouter = AudioRouter.shared
    
    override init() {
        super.init()
    }
    
    func initiateCall(to number: String) {
        guard let url = URL(string: "tel://\(number)") else { return }
        
        // Use UIApplication to trigger the native dialer
        // Note: For a true background dialer without UI, we'd need VoIP push + CallKit provider
        // which requires specific Apple entitlements. We use the URL scheme for this MVP.
        DispatchQueue.main.async {
            UIApplication.shared.open(url)
            self.audioRouter.routeToBluetoothHFP()
        }
    }
    
    func endCall() {
        // CXCallController can end calls if we manage them via CXProvider,
        // but since we used tel://, we cannot programmatically end it without private APIs.
        print("Cannot programmatically end native cellular call without Private APIs")
    }
    
    func answerCall() {
        print("Cannot programmatically answer native cellular call without Private APIs")
    }
}
