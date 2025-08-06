#!/bin/bash

# Sare Terminal and Shell Runner
# 
# This script provides easy access to run the Sare terminal emulator
# and shell components with proper environment setup.
# 
# Author: KleaSCM
# Email: KleaSCM@gmail.com
# File: run_sare.sh
# Description: Runner script for Sare terminal and shell components

echo "🚀 Sare Terminal & Shell Runner"
echo "💕 Built with love and passion by Yuriko and KleaSCM"
echo ""

# Function to run with clean environment
run_with_clean_env() {
	env -i PATH=/usr/local/bin:/usr/bin:/bin "$@"
}

# Check if we're in the right directory
if [ ! -d "sare-terminal" ] || [ ! -d "sare-shell" ]; then
	echo "❌ Error: Please run this script from the Sare project root directory"
	exit 1
fi

# Parse command line arguments
case "${1:-terminal}" in
	"terminal"|"t")
		echo "🖥️  Starting Sare Terminal Emulator..."
		cd sare-terminal
		run_with_clean_env cargo run
		;;
	"shell"|"s")
		echo "🐚 Starting Sare Shell..."
		cd sare-shell
		run_with_clean_env cargo run
		;;
	"build"|"b")
		echo "🔨 Building both components..."
		echo "Building terminal..."
		cd sare-terminal
		run_with_clean_env cargo build
		echo "Building shell..."
		cd ../sare-shell
		run_with_clean_env cargo build
		echo "✅ Build complete!"
		;;
	"clean"|"c")
		echo "🧹 Cleaning build artifacts..."
		cd sare-terminal
		run_with_clean_env cargo clean
		cd ../sare-shell
		run_with_clean_env cargo clean
		echo "✅ Clean complete!"
		;;
	"help"|"h"|"-h"|"--help")
		echo "Usage: $0 [command]"
		echo ""
		echo "Commands:"
		echo "  terminal, t    Run the terminal emulator (default)"
		echo "  shell, s       Run the shell component"
		echo "  build, b       Build both components"
		echo "  clean, c       Clean build artifacts"
		echo "  help, h        Show this help message"
		echo ""
		echo "Examples:"
		echo "  $0              # Run terminal emulator"
		echo "  $0 shell        # Run shell component"
		echo "  $0 build        # Build both components"
		;;
	*)
		echo "❌ Unknown command: $1"
		echo "Use '$0 help' for usage information"
		exit 1
		;;
esac 