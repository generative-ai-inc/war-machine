#!/bin/bash

# DEVELOPMENT SCRIPT
# This is a script to release war machine and copy the binary to the bin dir

cargo build --release

sudo cp target/release/wm /usr/local/bin/wm
