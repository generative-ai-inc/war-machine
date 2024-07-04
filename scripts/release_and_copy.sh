#!/bin/bash

# DEVELOPMENT SCRIPT
# This is a script to release war machine and copy the binary to the bin dir

cargo build --release

cp target/release/wm ~/bin/wm
