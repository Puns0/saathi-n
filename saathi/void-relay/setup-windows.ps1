# Requires Administrator privileges
Write-Host "Setting up VoidRelay for Windows..."

# Check Bluetooth service
$btService = Get-Service bthserv
if ($btService.Status -ne "Running") {
    Write-Host "Starting Bluetooth Support Service..."
    Set-Service bthserv -StartupType Automatic
    Start-Service bthserv
}

# Install Ollama
if (!(Get-Command ollama -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Ollama..."
    winget install --id Ollama.Ollama
} else {
    Write-Host "Ollama is already installed."
}

# Add firewall rule for port 7331
if (!(Get-NetFirewallRule -DisplayName "VoidRelay WebSocket" -ErrorAction SilentlyContinue)) {
    Write-Host "Adding Firewall Rule for Port 7331..."
    New-NetFirewallRule -DisplayName "VoidRelay WebSocket" -Direction Inbound -LocalPort 7331 -Protocol TCP -Action Allow
}

Write-Host "=========================================="
Write-Host "Setup Complete!"
Write-Host "You can now run 'cargo tauri dev' or 'cargo tauri build'."
Write-Host "=========================================="
