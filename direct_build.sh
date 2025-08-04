#!/bin/bash

echo "🌸 Direct build bypassing all proxies... (｡◕‿◕｡)"

# Completely unset all environment variables that might interfere
unset APPDIR LD_LIBRARY_PATH PERLLIB GSETTINGS_SCHEMA_DIR XDG_DATA_DIRS QT_PLUGIN_PATH CHROME_DESKTOP CURSOR_TRACE_ID GIT_ASKPASS VSCODE_GIT_ASKPASS_NODE VSCODE_GIT_ASKPASS_MAIN
unset RUSTC RUSTDOC CARGO
unset RUSTUP_TOOLCHAIN
unset RUST_BACKTRACE

# Set clean environment
export PATH="/usr/local/bin:/usr/bin:/usr/local/sbin:/usr/lib/rustup/bin"
export DISPLAY=":0"

echo "🔧 Building with clean environment..."
echo "Current PATH: $PATH"
echo "Current DISPLAY: $DISPLAY"

# Try to build
if cargo build --verbose; then
    echo "✅ Build successful!"
    echo "🚀 Running Sare GUI..."
    ./target/debug/sare
else
    echo "❌ Build failed"
    echo "💡 Trying alternative approach..."
    
    # Try with explicit rustc
    echo "🔧 Trying direct rustc compilation..."
    if /usr/lib/rustup/bin/rustc src/main.rs -o target/debug/sare_direct; then
        echo "✅ Direct compilation successful!"
        echo "🚀 Running direct build..."
        ./target/debug/sare_direct
    else
        echo "❌ Direct compilation also failed"
    fi
fi 