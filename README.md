# lazyflat

`lazyflat` is a simple, asynchronous terminal user interface (TUI) for managing Flatpak packages on Linux. Built with Rust, `ratatui`, and `tokio`, it provides a fast and responsive way to browse, install, update, and remove Flatpaks.

![lazyflat screenshot](https://raw.githubusercontent.com/username/lazyflat/main/screenshot.png) *(Placeholder for screenshot)*

## Features

- **Installed Apps**: View all your installed Flatpak applications.
- **Updates**: Check for and apply updates to individual apps or all of them at once.
- **Runtimes**: Manage installed runtimes separately.
- **Permissions Management**: View and toggle Flatpak permissions for any installed app.
- **Discover**: Search for new applications from remotes (like Flathub) and install them directly.
- **Async Backend**: Non-blocking operations ensure the UI remains responsive during long-running tasks.
- **Keyboard & Mouse Support**: Full support for both keyboard shortcuts and mouse interaction.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- `flatpak` CLI installed on your system.

### Build from Source

```bash
git clone https://github.com/yourusername/lazyflat.git
cd lazyflat
cargo build --release
```

The binary will be available at `target/release/lazyflat`.

## Usage

Run the application:

```bash
./target/release/lazyflat
```

### Keybindings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit / Close Help |
| `?` | Toggle Help |
| `/` | Enter Search Mode |
| `r` | Refresh Data |
| `h` / `Left` | Previous Tab |
| `l` / `Right` | Next Tab |
| `j` / `Down` | Next Item |
| `k` / `Up` | Previous Item |
| `p` | Open Permissions (Installed Apps) |
| `Space` | Toggle Permission (Permissions View) |
| `x` | Uninstall Selected |
| `u` | Update Selected |
| `U` | Update All |
| `i` | Install Selected (Discover Tab) |
| `Enter` | Confirm Search (Discover Tab) |

## Documentation

Full documentation is available in the `docs` directory. You can build it using Sphinx:

```bash
cd docs
make html
```

## License

This project is licensed under the MIT License.
