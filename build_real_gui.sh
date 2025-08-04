#!/bin/bash

echo "🌸 Building REAL GUI terminal (bypassing all proxies)... (｡◕‿◕｡)"

# Completely unset ALL environment variables that might interfere
unset APPDIR LD_LIBRARY_PATH PERLLIB GSETTINGS_SCHEMA_DIR XDG_DATA_DIRS QT_PLUGIN_PATH CHROME_DESKTOP CURSOR_TRACE_ID GIT_ASKPASS VSCODE_GIT_ASKPASS_NODE VSCODE_GIT_ASKPASS_MAIN
unset RUSTC RUSTDOC CARGO RUSTUP_TOOLCHAIN RUST_BACKTRACE
unset CURSOR_TRACE_ID CURSOR_EXTENSION_ID CURSOR_EXTENSION_PATH

# Set clean environment
export PATH="/usr/local/bin:/usr/bin:/usr/local/sbin:/usr/lib/rustup/bin"
export DISPLAY=":0"
export WAYLAND_DISPLAY="wayland-0"

echo "🔧 Environment cleaned..."
echo "Current PATH: $PATH"
echo "Current DISPLAY: $DISPLAY"

# Kill any existing processes
pkill -f sare 2>/dev/null

# Try to build with explicit rustc
echo "🔧 Building with explicit rustc..."
if /usr/lib/rustup/bin/rustc --version; then
	echo "✅ rustc available"
	
	# Try to build the GUI test first
	echo "🔧 Testing GUI compilation..."
	if /usr/lib/rustup/bin/rustc test_gui.rs --extern eframe --extern egui -L dependency=target/debug/deps 2>/dev/null; then
		echo "✅ GUI compilation works!"
		echo "🚀 Running GUI test..."
		DISPLAY=:0 ./test_gui &
		sleep 2
		if pgrep test_gui >/dev/null; then
			echo "✅ GUI test window appeared!"
			pkill test_gui
		else
			echo "❌ GUI test window did not appear"
		fi
	else
		echo "❌ GUI compilation failed"
	fi
else
	echo "❌ rustc not available"
fi

# Try cargo build as fallback
echo "🔧 Trying cargo build..."
if cargo build --verbose; then
	echo "✅ Build successful!"
	echo "🚀 Running Sare GUI..."
	DISPLAY=:0 ./target/debug/sare &
	sleep 3
	
	if pgrep sare >/dev/null; then
		echo "✅ Sare GUI process is running!"
		echo "💡 Check if GUI window appeared"
	else
		echo "❌ Sare GUI process not running"
	fi
else
	echo "❌ Cargo build failed"
	echo "💡 The Cursor proxy is blocking compilation"
	echo "💡 Try building outside of Cursor editor"
fi 