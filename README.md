# Sare Shell

A POSIX-compatible Rust shell with TUI interface built using `ratatui`.

## Features

- **Command Parsing**: Handle flags, quoting, escapes, and I/O redirection
- **Command Execution**: Execute external commands using `std::process::Command`
- **Built-in Commands**: `cd`, `exit`, `clear`, `history`, `pwd`, `echo`, `help`, `jobs`, `kill`
- **Job Control**: Background tasks, signal handling, PID tracking
- **TUI Interface**: Full terminal user interface with `ratatui`
- **Multipane Support**: Split terminal views like wezterm
- **Command History**: Persistent history with search
- **Autocomplete**: Tab completion for executables and files

## Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- Linux/Unix system

### Building and Running

```bash
# Clone the repository
git clone <repository-url>
cd sare

# Build the project
cargo build

# Run the shell
cargo run
```

### Cursor Editor Proxy Fix

If you're using Cursor editor and getting proxy errors, use our clean cargo wrapper:

```bash
# Use the clean wrapper (no Cursor proxy BS)
./cargo-clean build
./cargo-clean run

# Or use the build script
./build_sare.sh
```

The clean wrapper unsets all Cursor-related environment variables that interfere with the Rust toolchain.

## Architecture

```
src/
├── main.rs              # Application entry point
├── shell/               # Core shell logic
│   ├── parser.rs        # Command parsing
│   ├── executor.rs      # Command execution
│   ├── job.rs          # Job control
│   ├── builtins.rs     # Built-in commands
├── tui/                # Terminal UI
│   ├── layout.rs       # Layout management
│   ├── prompt.rs       # Input prompt
│   ├── output.rs       # Output display
│   ├── panes.rs        # Multipane support
├── config/             # Configuration
├── history/            # Command history
```

## Development

### Project Structure

- **Shell Core**: Command parsing, execution, and job management
- **TUI**: Terminal user interface using `ratatui`
- **Built-ins**: Internal shell commands
- **Job Control**: Background process management
- **History**: Persistent command history

### Testing

```bash
# Run tests
cargo test

# Run with clean environment
./cargo-clean test
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Author

**KleaSCM** - kleascm@gmail.com

Built with ❤️ and Rust 