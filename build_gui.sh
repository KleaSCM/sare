#!/bin/bash

# Build script for Sare GUI that bypasses Cursor proxy issues
echo "🌸 Building Sare GUI... (｡◕‿◕｡)"

# Temporarily unset Cursor-related environment variables
unset APPDIR LD_LIBRARY_PATH PERLLIB GSETTINGS_SCHEMA_DIR XDG_DATA_DIRS QT_PLUGIN_PATH CHROME_DESKTOP CURSOR_TRACE_ID GIT_ASKPASS VSCODE_GIT_ASKPASS_NODE VSCODE_GIT_ASKPASS_MAIN

# Set clean PATH
export PATH="/usr/local/bin:/usr/bin:/usr/local/sbin:/usr/lib/rustup/bin"

# Try to build
echo "🔧 Attempting to build Sare GUI..."
if cargo build; then
    echo "✅ Sare GUI built successfully!"
    echo "🚀 Running Sare GUI..."
    ./target/debug/sare
else
    echo "❌ Build failed due to proxy issues"
    echo "💡 Try running this script outside of Cursor editor"
    echo "💡 Or use: unset APPDIR LD_LIBRARY_PATH && cargo build"
fi 