#!/bin/bash
set -e

echo "Setting up VoidRelay for macOS..."

# Install Ollama via Homebrew
if ! command -v ollama &> /dev/null
then
    if ! command -v brew &> /dev/null
    then
        echo "Homebrew not found. Please install Homebrew first: https://brew.sh/"
        exit 1
    fi
    echo "Installing Ollama..."
    brew install --cask ollama
else
    echo "Ollama is already installed."
fi

echo "=========================================="
echo "Setup Complete!"
echo "Note: macOS requires you to manually grant Microphone and Bluetooth permissions when the app runs."
echo "You can now run 'cargo tauri dev' or 'cargo tauri build'."
echo "=========================================="
