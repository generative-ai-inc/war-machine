#!/bin/bash

set -u

abort() {
  printf "%s\n" "$@" >&2
  exit 1
}

echo "Installing War Machine to /usr/local/bin"

WM_PREFIX="/usr/local/bin"
WM_REPOSITORY="${WM_PREFIX}/wm"

OS="$(/usr/bin/uname -s)"
ARCH="$(/usr/bin/uname -m)"

DOWNLOAD_URL="https://github.com/gen-ai-inc/war-machine/releases/latest/download/wm-${ARCH}-${OS}"

curl -sSfL "${DOWNLOAD_URL}" -o "${WM_REPOSITORY}"
chmod +x "${WM_REPOSITORY}"
