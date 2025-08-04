#!/bin/bash

# Script to disable Cursor proxy interference with Rust
echo "🌸 Disabling Cursor proxy interference..."

# Method 1: Add proxy bypass to Cursor settings
echo "📝 Adding proxy bypass to Cursor settings..."
if [ -f ~/.config/Cursor/User/settings.json ]; then
	# Backup current settings
	cp ~/.config/Cursor/User/settings.json ~/.config/Cursor/User/settings.json.backup
	
	# Add proxy bypass settings
	cat >> ~/.config/Cursor/User/settings.json << 'EOF'
,
    "http.proxySupport": "off",
    "http.proxy": "",
    "http.proxyStrictSSL": false,
    "terminal.integrated.env.linux": {
        "RUSTUP_TOOLCHAIN": "stable-x86_64-unknown-linux-gnu"
    },
    "rust-analyzer.server.extraEnv": {
        "RUSTUP_TOOLCHAIN": "stable-x86_64-unknown-linux-gnu"
    }
EOF
	echo "✅ Added proxy bypass settings to Cursor"
else
	echo "❌ Cursor settings file not found"
fi

# Method 2: Create a clean environment script
echo "📝 Creating clean environment script..."
cat > ~/cursor_clean_env.sh << 'EOF'
#!/bin/bash
# Clean environment for building outside Cursor proxy

# Unset all Cursor-related environment variables
unset APPDIR LD_LIBRARY_PATH PERLLIB GSETTINGS_SCHEMA_DIR XDG_DATA_DIRS QT_PLUGIN_PATH CHROME_DESKTOP CURSOR_TRACE_ID GIT_ASKPASS VSCODE_GIT_ASKPASS_NODE VSCODE_GIT_ASKPASS_MAIN

# Set clean PATH
export PATH="/usr/local/bin:/usr/bin:/usr/local/sbin:/usr/lib/rustup/bin"

# Set clean environment
export RUSTUP_TOOLCHAIN="stable-x86_64-unknown-linux-gnu"
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"

echo "🌸 Clean environment ready for building!"
echo "💡 Use: source ~/cursor_clean_env.sh && cargo run"
EOF

chmod +x ~/cursor_clean_env.sh
echo "✅ Created clean environment script at ~/cursor_clean_env.sh"

# Method 3: Create a build script that bypasses proxy
echo "📝 Creating build script..."
cat > ~/build_sare_clean.sh << 'EOF'
#!/bin/bash

# Build Sare with clean environment (no Cursor proxy)
echo "🌸 Building Sare with clean environment..."

# Source clean environment
source ~/cursor_clean_env.sh

# Build the project
cd /home/klea/Documents/Dev/sare
cargo clean
cargo build

if [ $? -eq 0 ]; then
	echo "✅ Build successful!"
	echo "🚀 Running Sare GUI..."
	DISPLAY=:0 ./target/debug/sare
else
	echo "❌ Build failed"
	exit 1
fi
EOF

chmod +x ~/build_sare_clean.sh
echo "✅ Created build script at ~/build_sare_clean.sh"

echo ""
echo "🌸 Proxy disable complete! Here's what to do:"
echo ""
echo "1. Close Cursor editor completely"
echo "2. Open a regular terminal (not Cursor's terminal)"
echo "3. Run: ~/build_sare_clean.sh"
echo ""
echo "Or manually:"
echo "   source ~/cursor_clean_env.sh"
echo "   cd /home/klea/Documents/Dev/sare"
echo "   cargo run"
echo ""
echo "This should build the GUI without proxy interference! 💕" 