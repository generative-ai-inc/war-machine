#!/bin/bash

abort() {
  printf "%s\n" "$@" >&2
  exit 1
}

echo "Installing War Machine to /usr/local/bin"

WM_DIR="/usr/local/bin"
WM_PATH="${WM_DIR}/wm"

OS="$(/usr/bin/uname -s)"
ARCH="$(/usr/bin/uname -m)"

# Make os lowercase
OS="$(echo ${OS} | tr '[:upper:]' '[:lower:]')"
ASSET_NAME="war-machine-${ARCH}-${OS}"

# Get the latest release information
RELEASE_INFO=$(curl -fsSL https://api.github.com/repos/generative-ai-inc/war-machine/releases/latest)

# # Extract the asset download URL
ASSET_URL=$(echo $RELEASE_INFO | python3 -c "import sys, json; data = json.load(sys.stdin); print(next(asset['url'] for asset in data['assets'] if asset['name'] == '$ASSET_NAME'))")

# Check if the asset URL is found
if [ -z "$ASSET_URL" ]; then
  echo "Asset not found: $ASSET_NAME"
  exit 1
fi

# Add accepts header to the request
sudo curl -H "Accept: application/octet-stream" -fsSL "${ASSET_URL}" -o "${WM_PATH}"

sudo chmod +x "${WM_PATH}"
echo "🔫 War Machine installed. Run 'wm' to get started."

# Previously, we used to install to /usr/local/bin/war-machine and /usr/local/bin/war
# We remove these files, if they exist, to avoid confusion
sudo rm -f /usr/local/bin/war-machine
sudo rm -f /usr/local/bin/war
