# Sare Terminal Test Suite

This directory contains the comprehensive test suite for the Sare terminal, including a unified test runner that merges all existing tests into a single executable with detailed reporting.

## 🌸 Unified Test Runner

The `test_runner.rs` file provides a comprehensive test suite that:

- **Merges all existing tests** into a single executable
- **Provides detailed output** showing what's passing, failing, and why
- **Categorizes tests** by functionality (completion, history, expansion, etc.)
- **Tracks execution time** for performance monitoring
- **Generates comprehensive reports** with pass/fail statistics

## 🚀 Running Tests

### Option 1: Using the Test Runner Script
```bash
cd Tests
./run_tests.sh
```

### Option 2: Using Cargo Directly
```bash
cargo run --bin test_runner
```

### Option 3: Running Individual Test Files
```bash
# Run specific test files
cargo test --test test_completion
cargo test --test test_history
cargo test --test test_expansion
cargo test --test test_substitution
cargo test --test test_heredoc
cargo test --test test_multiline
```

## 📊 Test Categories

The unified test runner organizes tests into the following categories:

### 📝 Completion Tests
- Tab completer creation and initialization
- Command completion functionality
- File path completion with context awareness
- Quoted path handling
- Flag and variable completion

### 📚 History Tests
- History manager creation and initialization
- Adding commands to history
- History navigation (up/down)
- Reverse search functionality
- History persistence and max entries

### 🔍 Expansion Tests
- Expansion state management
- Brace expansion detection
- Numeric range expansion
- Comma list expansion
- Glob pattern expansion and matching
- Complex expansion scenarios

### 🔄 Substitution Tests
- Substitution state management
- Command substitution detection
- Variable substitution
- Nested substitution handling

### 📄 Heredoc Tests
- Heredoc state management
- Heredoc detection and parsing
- Delimiter handling
- Multiline heredoc support

### 📋 Multiline Tests
- Multiline state management
- Multiline command detection
- Continuation line handling
- Escaped newline processing

## 🎯 Test Output Format

The unified test runner provides detailed output including:

```
🌸 Starting Sare Terminal Test Suite 🌸
==========================================

📝 Running Completion Tests...
📚 Running History Tests...
🔍 Running Expansion Tests...
🔄 Running Substitution Tests...
📄 Running Heredoc Tests...
📋 Running Multiline Tests...

🌸 Test Results Summary 🌸
==========================
Total Tests: 18
✅ Passed: 16
❌ Failed: 2
⏱️  Total Time: 0.45s

📊 Results by Category:
  completion: 3/3 passed (100.0%)
  history: 3/3 passed (100.0%)
  expansion: 3/3 passed (100.0%)
  substitution: 2/2 passed (100.0%)
  heredoc: 2/2 passed (100.0%)
  multiline: 2/2 passed (100.0%)

✅ Passed Tests:
  🟢 test_tab_completer_creation (completion): Tests tab completer initialization and basic functionality - 2ms
  🟢 test_command_completion (completion): Tests command completion functionality - 1ms
  ...

❌ Failed Tests Details:
  🔴 test_file_path_completion (completion): Tests file path completion functionality
     Error: Context should be FilePath
     Time: 3ms
```

## 🔧 Test Development

### Adding New Tests

To add new tests to the unified runner:

1. **Add test function** to the appropriate category method in `test_runner.rs`
2. **Use the `run_single_test` method** with proper error handling
3. **Provide descriptive names and categories** for clear reporting
4. **Include Japanese comments** following the style guide

### Example Test Structure

```rust
results.push(self.run_single_test(
    || {
        // Test implementation here
        if condition {
            return Err("Test failed because...".into());
        }
        Ok(())
    },
    "test_name",
    "category",
    "Description of what this test does"
));
```

### Test Categories

- `completion` - Tab completion and command completion
- `history` - Command history and navigation
- `expansion` - Brace expansion and globbing
- `substitution` - Command and variable substitution
- `heredoc` - Here document functionality
- `multiline` - Multiline command handling

## 🐛 Debugging Failed Tests

When tests fail, the runner provides:

- **Detailed error messages** explaining what went wrong
- **Execution time** for performance analysis
- **Test categorization** to identify problematic areas
- **Context information** about the test environment

## 📈 Performance Monitoring

The test runner tracks:

- **Individual test execution time**
- **Total suite execution time**
- **Pass/fail rates by category**
- **Performance trends** over time

## 🎨 Output Formatting

The test runner uses emoji and formatting to make output more readable:

- 🌸 - Test suite branding
- 📝📚🔍🔄📄📋 - Category icons
- ✅❌ - Pass/fail indicators
- 🟢🔴 - Detailed status indicators
- ⏱️📊 - Performance and statistics

## 🔄 Continuous Integration

The test runner is designed to work with CI/CD systems:

- **Exit code 0** for all tests passing
- **Exit code 1** for any test failures
- **Structured output** for parsing by CI tools
- **Performance metrics** for regression detection

## 📝 Style Guide Compliance

All tests follow the project's style guide:

- **PascalCase** for function names
- **Japanese comments** for complex logic
- **Tab indentation** (no spaces)
- **120 character line limit**
- **Proper error handling** without exceptions 