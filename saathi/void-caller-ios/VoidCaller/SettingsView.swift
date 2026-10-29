import SwiftUI

struct SettingsView: View {
    @Environment(\.presentationMode) var presentationMode
    @EnvironmentObject var appState: AppState

    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Connection Settings")) {
                    HStack {
                        Text("PC IP Address")
                        Spacer()
                        TextField("192.168.1.100", text: $appState.serverIp)
                            .multilineTextAlignment(.trailing)
                            .keyboardType(.decimalPad)
                    }
                    HStack {
                        Text("Port")
                        Spacer()
                        TextField("7331", text: $appState.serverPort)
                            .multilineTextAlignment(.trailing)
                            .keyboardType(.numberPad)
                    }
                }

                Section(header: Text("Permissions"), footer: Text("VoidCaller requires background permissions and CallKit integration to function properly.")) {
                    Text("Permissions are requested automatically.")
                        .foregroundColor(.gray)
                }
            }
            .navigationTitle("Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        presentationMode.wrappedValue.dismiss()
                    }
                }
            }
        }
        .preferredColorScheme(.dark)
    }
}
