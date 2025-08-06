#!/bin/bash

# Working Sare Terminal Runner
#
# This script runs the Working Sare terminal with proper environment variables
# and ALL the amazing features from the original 40,000+ line codebase!
#
# Author: KleaSCM
# Email: KleaSCM@gmail.com
# File: run_working_terminal.sh
# Description: Runner script for Working Sare terminal with ALL features

echo "🚀 Starting Working Sare Terminal Emulator..."
echo "💕 Built with love and passion by Yuriko and KleaSCM"
echo "✨ Features: Multi-pane, GPU acceleration, History navigation, Tab completion"
echo "🎯 ANSI support, Unicode, Bidirectional text, Image support, Hyperlinks"
echo "🔍 Semantic highlighting, Search, Selection, Paste protection, Plugins"

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

# Run the terminal with clean environment but preserve display
env -i PATH=/usr/local/bin:/usr/bin:/bin DISPLAY=:0 XDG_SESSION_TYPE=x11 cargo run 