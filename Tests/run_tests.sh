#!/bin/bash

/**
 * Test runner script for Sare terminal
 * 
 * This script provides an easy way to run the unified test suite
 * with proper output formatting and error handling.
 * 
 * Author: KleaSCM
 * Email: KleaSCM@gmail.com
 * File: run_tests.sh
 * Description: Test runner script with comprehensive output
 */

echo "🌸 Sare Terminal Test Runner 🌸"
echo "================================"
echo ""

# Check if we're in the right directory
if [ ! -f "test_runner.rs" ]; then
	echo "❌ Error: test_runner.rs not found in current directory"
	echo "Please run this script from the Tests directory"
	exit 1
fi

# Compile and run the test runner
echo "🔧 Compiling test runner..."
if cargo run --bin test_runner; then
	echo ""
	echo "✅ Test runner completed successfully!"
	exit 0
else
	echo ""
	echo "❌ Test runner failed!"
	exit 1
fi 