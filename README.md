# Super V

> **Note**: Auto-generated

A modern clipboard manager for Linux that maintains a persistent history of your clipboard items, inspired by Windows 11's clipboard experience.

## Overview

**Super V** is a lightweight, daemon-based clipboard manager written in Rust. It monitors your system clipboard in real-time, maintains a history of copied items (both text and images), and provides IPC-based communication for managing clipboard history programmatically.

## Features

- **Clipboard History**: Maintains up to 25 clipboard items (text and images)
- **Real-time Monitoring**: Automatically detects and saves new clipboard content
- **Multi-format Support**: Handles both text and image clipboard data
- **Duplicate Detection**: Automatically promotes existing items instead of creating duplicates
- **Daemon Service**: Runs as a background systemd user service
- **IPC Communication**: Unix socket-based IPC for external control
- **Thread-safe**: Concurrent polling and command handling with mutex-protected shared state
- **Single Instance**: Process locking prevents multiple daemon instances

## Architecture

### Core Components

1. **Clipboard Manager** (`clipboard_manager.rs`)

   - Manages two concurrent services:
     - **Polling Service**: Monitors system clipboard every 500ms
     - **Command Service**: Listens for IPC commands via Unix socket
   - Thread-safe access to shared clipboard history
2. **Clipboard History** (`history.rs`)

   - Fixed-size queue (VecDeque) implementation
   - Smart duplicate handling with auto-promotion
   - Serializable for IPC transmission
3. **IPC Server** (`clipboard_ipc_server.rs`)

   - Unix domain socket at `LOCK_PATH` (check common.rs)
   - MessagePack serialization for efficient data transfer
   - Supports commands: Snapshot, Promote, Delete, Clear

## Installation

### Prerequisites

- Rust toolchain (2024 edition)
- Linux system with systemd
- X11 or Wayland session

> **Note**: This should theoretically work on Wayland as well since the packages used support wayland. This is being developed on XWayland and is mainly meant for X11/XWayland.

### Build and Install

```bash
# Build release binary
cargo build --release

# Install as systemd user service
chmod +x install.sh
./install.sh

# Alternatively, install as dpkg (system-wide. Needs sudo)
strip target/release/super_v
cargo deb --no-build
sudo dpkg -i ./target/debian/super-v_0.1.0-1_amd64.deb 
```

The install script will:

- Build the project in release mode
- Copy the binary to `/usr/local/bin/super_v`
- Create a systemd user service at `~/.config/systemd/user/super_v.service`
- Enable and start the service
- Configure automatic restart on failure

## Usage

### Command Line Interface

```bash
# Needs sudo if installed through dpkg

# Start the daemon manually (usually handled by systemd)
super_v start

# Open GUI (GTK-based, in development)
super_v open-gui

# Clean up stale socket and lock files
super_v clean
```

### IPC Commands

The daemon accepts the following commands via Unix socket (`SOCKET_PATH` - check common.rs):

#### Commands (CmdIPC)

- **Snapshot**: Retrieve the current clipboard history
- **Promote(pos)**: Move item at position `pos` to the front
- **Delete(pos)**: Remove item at position `pos`
- **Clear**: Clear entire clipboard history

#### Response (IPCResponse)

```rust
pub struct IPCResponse {
    pub history_snapshot: Option<ClipboardHistory>,
    pub message: Option<String>
}
```

### Example: Using IPC from Rust

```rust
use super_v::services::clipboard_ipc_server::{
    create_default_stream,
    send_payload,
    read_payload,
    Payload,
    CmdIPC
};

fn main() {
    // Connect to daemon
    let mut stream = create_default_stream().unwrap();
  
    // Request clipboard snapshot
    send_payload(&mut stream, Payload::Cmd(CmdIPC::Snapshot));
  
    // Read response
    let response = read_payload(&mut stream);
    println!("{:?}", response);
}
```

## Configuration

### Service Configuration

The systemd service is configured at `~/.config/systemd/user/super_v.service`:

- **Type**: Simple
- **Restart Policy**: Restart on failure after 5 seconds
- **Logs**: Appended to `~/superv.log`
- **Environment Variables**:
  - `RUST_BACKTRACE=1`
  - `RUST_LOG=info`
  - `DISPLAY=:0`

### Clipboard History Size

Default maximum history size is **25 items**. To change this, modify `CLIPBOARD_SIZE` in `src/services/clipboard_manager.rs`:

```rust
const CLIPBOARD_SIZE: usize = 25;  // Change this value
```

## Uninstallation

```bash
chmod +x uninstall.sh
./uninstall.sh
```

This will:

- Stop and disable the systemd service
- Remove service files
- Remove the binary from `/usr/local/bin`
- Clean up log files
- Remove socket and lock files

## Development

### Project Structure

```
super_v/
├── src/
│   ├── main.rs                     # CLI entry point
│   ├── lib.rs                      # Library root
│   ├── common.rs                   # Shared types and errors
│   ├── history.rs                  # ClipboardHistory implementation
│   ├── services/
│   │   ├── mod.rs
│   │   ├── clipboard_manager.rs    # Main daemon manager
│   │   └── clipboard_ipc_server.rs # IPC communication
│   ├── gui/                        # (Future: Add GUI)
│   └── ydotol.rs                   # (Future: keyboard simulation)
├── tests/
│   ├── history_test.rs
│   └── manager_test.rs
├── Cargo.toml
├── install.sh
├── uninstall.sh
└── README.md
```

### Running Tests

```bash
# Run all tests
cargo test
```

## Roadmap

### Planned Features

- [ ] **GUI Interface**: GTK-based clipboard history viewer (in progress)
- [ ] **Keyboard Simulation**: Auto-paste selected items using ydotool/rdev
- [ ] **Search & Filter**: Search through clipboard history
- [ ] **Persistent Storage**: Save history across reboots
- [ ] **Emoji Picker**: Quick emoji insertion

### Known Issues

- Uses `unwrap()` in several places (should be replaced with proper error handling)
- No persistent storage (history lost on daemon restart)
- Limited to Unix-like systems (uses Unix domain sockets)

Here’s the improved, professional version of the  **License** ,  **Author** , and **Contributing** sections — aligned with open-source best practices and phrased for real-world GitHub use.

---

## Contributing

Contributions are appreciated. You can help improve Super V in many ways:

### How to Contribute

1. **Fork the repository**
   * Click the “Fork” button on GitHub to create your own copy.
2. **Clone your fork**
   ```bash
   git clone https://github.com/<your-username>/super_v.git
   cd super_v
   ```
3. **Create a new branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Make your changes**
   * Follow Rust’s standard formatting rules:
     ```bash
     cargo fmt
     ```
   * Run the tests:
     ```bash
     cargo test
     ```
5. **Commit and push**
   ```bash
   git add .
   git commit -m "Add <short description>"
   git push origin feature/your-feature-name
   ```
6. **Open a Pull Request**
   * Go to your fork on GitHub and click  **New Pull Request** .
   * Describe the change clearly — why you made it, and what it improves.

### Contribution Areas

* Fixing error handling (`Result` instead of `unwrap`)
* Building and improving the GTK-based GUI
* Adding persistent history storage
* Extending format detection (HTML, RTF, etc.)
* Improving test coverage
* Optimizing IPC performance and serialization

### Guidelines

* Keep commits atomic and clear.
* Document new functions or modules.
* Ensure the binary builds in release mode (`cargo build --release`).
* Avoid adding unnecessary dependencies
