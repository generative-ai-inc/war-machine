#!/bin/bash

abort() {
  printf "%s\n" "$@" >&2
  exit 1
}

echo "Installing War Machine to /usr/local/bin"

GITHUB_TOKEN=""
while [ "$GITHUB_TOKEN" == "" ]; do
  read -sp "Enter your GITHUB_TOKEN:" GITHUB_TOKEN
  echo
done

# Must be run with sudo
if [ "$(id -u)" -ne 0 ]; then
  abort "This script must be run with sudo"
fi

WM_DIR="/usr/local/bin"
WM_PATH="${WM_DIR}/wm"

OS="$(/usr/bin/uname -s)"
ARCH="$(/usr/bin/uname -m)"

# Make os lowercase
OS="$(echo ${OS} | tr '[:upper:]' '[:lower:]')"
ASSET_NAME="wm-${ARCH}-${OS}"

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
curl -H "Authorization: bearer ${GITHUB_TOKEN}" -H "Accept: application/octet-stream" -fsSL "${ASSET_URL}" -o "${WM_PATH}"

chmod +x "${WM_PATH}"

echo "🔫 War Machine installed. Run 'wm' to get started."

${WM_PATH} token add GITHUB_TOKEN ${GITHUB_TOKEN}
