#!/bin/bash

# Sare Terminal Runner
# 
# This script runs the Sare terminal with proper environment variables
# 
# Author: KleaSCM
# Email: KleaSCM@gmail.com
# File: run_terminal.sh
# Description: Runner script for Sare terminal with proper environment

echo "🚀 Starting Sare Terminal Emulator..."
echo "💕 Built with love and passion by Yuriko and KleaSCM"

# Ensure we have the display environment
export DISPLAY=:0
export XDG_SESSION_TYPE=x11

# Clean problematic variables but keep essential ones
unset CURSOR_TRACE_ID
unset CURSOR_DEVICE_ID
unset CURSOR_SESSION_ID

echo "🖼️  Starting GUI terminal window..."
echo "📺 Display: $DISPLAY"
echo "🖥️  Session Type: $XDG_SESSION_TYPE"

# Run the terminal
cargo run
