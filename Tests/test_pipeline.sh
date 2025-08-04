#!/bin/bash

echo "Testing Sare Shell Pipeline Features"
echo "==================================="
echo ""

echo "1. Testing Pipes (|):"
echo "ls | grep .txt"
echo ""

echo "2. Testing Command Chaining (&&):"
echo "echo 'First command' && echo 'Second command'"
echo ""

echo "3. Testing Command Chaining (||):"
echo "false || echo 'This should run'"
echo ""

echo "4. Testing Sequential Commands (;):"
echo "echo 'Command 1'; echo 'Command 2'; echo 'Command 3'"
echo ""

echo "5. Testing Complex Pipeline:"
echo "ls | grep .rs && echo 'Found Rust files' || echo 'No Rust files found'"
echo ""

echo "6. Testing Real-time Output:"
echo "ping -c 3 localhost"
echo ""

echo "Ready to test in Sare shell!" 