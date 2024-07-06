#!/bin/bash

abort() {
  printf "%s\n" "$@" >&2
  exit 1
}

echo "Installing War Machine to /usr/local/bin"

GITHUB_TOKEN=""
read -sp "Enter your GITHUB_TOKEN:" GITHUB_TOKEN
echo

WM_DIR="/usr/local/bin"
WM_PATH="${WM_DIR}/war-machine"

OS="$(/usr/bin/uname -s)"
ARCH="$(/usr/bin/uname -m)"

# Make os lowercase
OS="$(echo ${OS} | tr '[:upper:]' '[:lower:]')"
ASSET_NAME="war-machine-${ARCH}-${OS}"

# Get the latest release information
RELEASE_INFO=$(curl -H "Authorization: bearer ${GITHUB_TOKEN}" -fsSL https://api.github.com/repos/generative-ai-inc/war-machine/releases/latest)

# Extract the asset download URL
ASSET_URL=$(echo $RELEASE_INFO | jq -r ".assets[] | select(.name == \"$ASSET_NAME\") | .url")

# Check if the asset URL is found
if [ -z "$ASSET_URL" ]; then
  echo "Asset not found: $ASSET_NAME"
  exit 1
fi

# Add accepts header to the request
sudo curl -H "Authorization: bearer ${GITHUB_TOKEN}" -H "Accept: application/octet-stream" -fsSL "${ASSET_URL}" -o "${WM_PATH}"

# Symlink the war-machine command to war
sudo ln -sf "${WM_PATH}" "${WM_DIR}/war"

# Symlink the war-machine command to wm
sudo ln -sf "${WM_PATH}" "${WM_DIR}/wm"

sudo chmod +x "${WM_PATH}"
echo "🔫 War Machine installed. Run 'war-machine' to get started."

${WM_PATH} secret add GITHUB_TOKEN ${GITHUB_TOKEN}
