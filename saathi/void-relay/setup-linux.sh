#!/bin/bash
set -e

echo "Setting up VoidRelay for Linux..."

# Install dependencies
sudo apt-get update
sudo apt-get install -y pulseaudio pulseaudio-module-bluetooth bluez wget curl ufw

# Enable bluetooth service
sudo systemctl enable bluetooth
sudo systemctl start bluetooth

# Add user to bluetooth and audio groups
sudo usermod -aG bluetooth $USER
sudo usermod -aG audio $USER

# Install Ollama
if ! command -v ollama &> /dev/null
then
    echo "Installing Ollama..."
    curl -fsSL https://ollama.com/install.sh | sh
else
    echo "Ollama is already installed."
fi

# Set firewall rule for port 7331
sudo ufw allow 7331/tcp

echo "=========================================="
echo "Setup Complete!"
echo "Please restart your computer or log out and log back in for group changes to take effect."
echo "You can now run 'cargo tauri dev' or 'cargo tauri build'."
echo "=========================================="
