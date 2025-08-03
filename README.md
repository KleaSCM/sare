# Sare Shell

shell with full TUI experience built using `ratatui`.

## Features

### Core Shell Functionality
- **Advanced Command Parsing**: Full shell command parsing with argument splitting, quoting, and escape sequences
- **Comprehensive Built-in Commands**: 50+ built-in commands including filesystem, text processing, system, and development tools
- **Job Control**: Complete background job management with `fg`, `bg`, `jobs`, `kill` commands
- **Signal Handling**: Proper handling of Ctrl+C (SIGINT), Ctrl+Z (SIGTSTP) signals

### User Interface
- **Beautiful TUI**: Modern terminal user interface built with `ratatui`
- **Tab Completion**: Intelligent autocomplete for commands and file paths
- **Command History**: Persistent command history with search and navigation
- **Real-time Output**: Commands display output as they execute

### Advanced Features
- **Pipeline Support**: Full pipeline support with pipes (`|`) for command chaining
- **Command Chaining**: Logical AND (`&&`), OR (`||`), and sequential (`;`) operators
- **I/O Redirection**: Support for input/output redirection (`>`, `<`, `>>`)
- **Environment Variables**: Full environment variable support and expansion (`$PATH`, `$HOME`, etc.)
- **Multi-pane Support**: Split terminal views like wezterm (planned)

## 🎯 Quick Start

```bash
# Build the shell
cargo build

# Run the shell
cargo run
```

## 📖 Usage Examples

### Basic Commands
```bash
ls -la                    # List files with details
cd ~/Documents           # Change directory
pwd                      # Print working directory
echo "Hello, World!"     # Echo text
cat file.txt             # Display file contents
grep "pattern" file.txt  # Search for patterns
```

### Filesystem Operations
```bash
mkdir new_directory      # Create directory
cp source dest          # Copy files
mv old_name new_name    # Move/rename files
rm file.txt             # Remove files
touch new_file.txt      # Create empty file
```

### Pipeline Operations
```bash
ls | grep .txt          # Pipe output through grep
cat file.txt | wc -l    # Count lines in file
ps aux | grep python    # Find Python processes
find . -name "*.rs" | head -5  # Find Rust files, show first 5
```

### Command Chaining
```bash
echo "First" && echo "Second"    # Run second only if first succeeds
false || echo "This runs"        # Run second only if first fails
ls; echo "Done"; pwd             # Run commands sequentially
mkdir test && cd test && touch file.txt  # Create dir, enter it, create file
```

### I/O Redirection
```bash
echo "Hello" > output.txt        # Redirect output to file
cat < input.txt                  # Redirect input from file
echo "Append" >> output.txt      # Append to file
ls -la 2> errors.log            # Redirect stderr to file
```

### Environment Variables
```bash
echo $PATH                       # Display PATH variable
export MY_VAR="value"           # Set environment variable
echo $HOME                       # Display home directory
```

### Job Control
```bash
sleep 10 &                      # Run command in background
jobs                            # List background jobs
fg %1                          # Bring job 1 to foreground
kill %1                        # Kill background job
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Sare Shell Architecture                 │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                     TUI Layer (ratatui)                   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │
│  │   Layout    │ │   Prompt    │ │   Output    │        │
│  │  Manager    │ │  Manager    │ │  Manager    │        │
│  └─────────────┘ └─────────────┘ └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Shell Core Layer                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │
│  │   Parser    │ │  Executor   │ │ Job Manager │        │
│  │ (Pipeline)  │ │(Real-time)  │ │ (Signals)   │        │
│  └─────────────┘ └─────────────┘ └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                   Command System Layer                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │
│  │ Filesystem  │ │    Text     │ │   System    │        │
│  │ Commands    │ │ Processing  │ │  Commands   │        │
│  └─────────────┘ └─────────────┘ └─────────────┘        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │
│  │   Process   │ │  Network    │ │Development  │        │
│  │ Commands    │ │ Commands    │ │ Commands    │        │
│  └─────────────┘ └─────────────┘ └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                   Support Layer                           │
│  ┌─────────────┐ ┌─────────────┐                        │
│  │   History   │ │   Config    │                        │
│  │  Manager    │ │  Manager    │                        │
│  └─────────────┘ └─────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

### Core Modules
- **`src/shell/`**: Core shell logic (parsing, execution, job control)
  - `parser.rs`: Command parsing with pipeline support
  - `executor.rs`: Command execution with real-time output
  - `job.rs`: Background job management and signal handling
  - `mod.rs`: Main shell state and coordination

### User Interface
- **`src/tui/`**: Terminal user interface components
  - `mod.rs`: Main TUI rendering and layout
  - `layout.rs`: Layout management and constraints
  - `prompt.rs`: Command input and prompt handling
  - `output.rs`: Output display and formatting
  - `panes.rs`: Multi-pane support (planned)

### Command System
- **`src/commands/`**: Comprehensive built-in command implementations
  - `filesystem.rs`: cd, ls, pwd, mkdir, rm, cp, mv, touch
  - `text.rs`: echo, cat, grep, sed, awk, sort, uniq, wc
  - `system.rs`: exit, clear, history, help, alias, export, env
  - `process.rs`: jobs, kill, bg, fg, wait
  - `network.rs`: ping, curl, wget, netstat
  - `development.rs`: git, cargo, make, npm

### Support Modules
- **`src/history/`**: Command history management with persistence
- **`src/config/`**: Configuration and settings management

## 📁 File Structure

```
sare/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── shell/                  # Core shell functionality
│   │   ├── mod.rs             # Main shell state and coordination
│   │   ├── parser.rs          # Command parsing with pipeline support
│   │   ├── executor.rs        # Command execution with real-time output
│   │   ├── job.rs             # Job management and signal handling
│   │   ├── builtins.rs        # Legacy built-in commands
│   │   └── commands/          # Comprehensive command system
│   │       ├── mod.rs         # Command registry and handlers
│   │       ├── filesystem.rs  # Filesystem operations (cd, ls, pwd, etc.)
│   │       ├── text.rs        # Text processing (echo, cat, grep, etc.)
│   │       ├── system.rs      # System commands (exit, clear, history, etc.)
│   │       ├── process.rs     # Process management (jobs, kill, bg, fg)
│   │       ├── network.rs     # Network utilities (ping, curl, wget)
│   │       └── development.rs # Development tools (git, cargo, make, npm)
│   ├── tui/                   # Terminal user interface
│   │   ├── mod.rs             # Main TUI rendering and layout
│   │   ├── layout.rs          # Layout management and constraints
│   │   ├── prompt.rs          # Command input and prompt handling
│   │   ├── output.rs          # Output display and formatting
│   │   └── panes.rs           # Multi-pane support (planned)
│   ├── history/               # Command history management
│   │   └── mod.rs             # History persistence and search
│   └── config/                # Configuration management
│       └── mod.rs             # Settings and configuration handling
├── Tests/                     # Test suite
├── Docs/                      # Project documentation
├── Cargo.toml                 # Project dependencies and metadata
├── .gitignore                 # Git ignore patterns
├── README.md                  # This file
├── test_pipeline.sh           # Pipeline testing examples
└── cargo-clean               # Build wrapper script
```

## 🔧 Development

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

