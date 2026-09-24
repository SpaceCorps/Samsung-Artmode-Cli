# Samsung Art Mode CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/Samsung-Artmode-Cli?color=blue&label=version)](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/Samsung-Artmode-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Samsung-Artmode-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/Samsung-Artmode-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A blazing fast, native command-line tool and agent interface for **Samsung The Frame TV Art Mode**. Built in native Rust 2024 for developers, home automation enthusiasts, and autonomous AI agents.

---

## Highlights

- ⚡ **Sub-3ms Startup**: Compiled as a native static binary with zero runtime dependencies. Cold-starts in ~1–3 ms.
- 🔐 **OS Keystore Integration**: TV authentication tokens are stored securely in native host vaults (macOS Keychain, Linux Secret Service, Windows DPAPI).
- 📺 **Full Art Mode Surface**: Commands for pairing, status, artwork listing, current display, selection, image uploads, deletion, favorites, brightness adjustment, matte swapping, and slideshow automation.
- 🤖 **AI Agent Native**: Machine-readable `--json` output, standardized error envelopes with stable exit codes, and self-documenting `agent-readme`.
- 🛡️ **Multi-TV Scoping**: Named TV accounts (e.g. `living-room`, `bedroom`) with explicit parameter scoping prevent accidental display mutations across rooms.
- 🏠 **100% Local Network**: Communicates directly over local WebSocket (ports 8001/8002) and TCP sockets with zero third-party cloud dependence.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/Samsung-Artmode-Cli --locked
```

### Pre-built Standalone Binaries

Download standalone binary archives directly from the [GitHub Releases](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/latest) page:

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`samsung-artmode-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/download/v1.0.0/samsung-artmode-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`samsung-artmode-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/download/v1.0.0/samsung-artmode-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`samsung-artmode-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/download/v1.0.0/samsung-artmode-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Linux** | ARM64 (musl static) | [`samsung-artmode-v1.0.0-aarch64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/download/v1.0.0/samsung-artmode-v1.0.0-aarch64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`samsung-artmode-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/Samsung-Artmode-Cli/releases/download/v1.0.0/samsung-artmode-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Quickstart

### 1. Pair with your TV

Make sure your Samsung The Frame TV is turned on and connected to the same local network.

```bash
# Auto-discover TV via SSDP and save as 'living-room'
samsung-artmode pair --name living-room

# Or target directly by IP address
samsung-artmode pair 192.168.1.100 --name living-room
```

Press **Allow** on your TV screen when prompted. The returned auth token is stored automatically in your operating system's native keystore.

### 2. Check Status & Browse Artwork

```bash
# Check if Art Mode is active
samsung-artmode status -a living-room

# View currently displayed artwork
samsung-artmode current -a living-room

# List photos uploaded to the TV
samsung-artmode list --category MY-C0002 -a living-room
```

### 3. Upload & Display Art

```bash
# Upload JPEG or PNG photo, apply a polar matte, and display immediately
samsung-artmode upload family-photo.jpg --matte flexible_polar --select -a living-room

# Change the matte on existing artwork
samsung-artmode matte set SAM-F0206 shadowbox_black -a living-room

# Set display brightness (0-10)
samsung-artmode brightness set 7 -a living-room
```

### 4. Slideshow Automation

```bash
# Start a 30-minute rotating shuffled slideshow of your personal photos
samsung-artmode slideshow --interval 30 --shuffle -a living-room

# Turn off slideshow
samsung-artmode slideshow --off -a living-room
```

---

## Command Reference

Every command that communicates with a TV accepts `--account <name>` (short `-a <name>`) or `--host <ip>`.

### Pairing & Multi-TV Management

| Command | Description |
|:---|:---|
| `samsung-artmode pair [host] [--name <name>]` | Auto-discover or target TV by IP, approve pairing on screen, and save token |
| `samsung-artmode accounts list` | List all configured TV accounts and their IP hosts |
| `samsung-artmode accounts add <name> --host <ip>` | Manually register a TV account in config |
| `samsung-artmode accounts test <name>` | Verify connectivity and stored token validity |
| `samsung-artmode accounts remove <name> [--yes]` | Delete account and purge token from OS keystore |

### Art Mode Control

| Command | Description |
|:---|:---|
| `samsung-artmode status` | Get Art Mode status (`{"status": "on"}` or `{"status": "off"}`) |
| `samsung-artmode list [--category <cat>]` | List artwork on TV (`MY-C0002`=Photos, `MY-C0004`=Favorites, `MY-C0008`=Store) |
| `samsung-artmode current` | Get metadata for currently displayed artwork |
| `samsung-artmode select <id> [--show]` | Select and display an artwork by content ID |
| `samsung-artmode upload <file> [--matte <id>] [--select]` | Upload PNG or JPEG image over binary TCP socket |
| `samsung-artmode delete <id>` | Delete an image from the TV |
| `samsung-artmode favorite <id> [--remove]` | Add or remove an image from Favorites |
| `samsung-artmode brightness get` | Get display brightness level (0–10) |
| `samsung-artmode brightness set <0-10>` | Set display brightness level |
| `samsung-artmode matte list` | List available frame matte styles |
| `samsung-artmode matte set <id> <matte_id>` | Apply a frame matte to an image |
| `samsung-artmode slideshow [--interval <m>] [--shuffle]` | Configure or disable slideshow rotation |
| `samsung-artmode agent-readme [--json]` | Self-documenting operating manual for autonomous agents |

---

## AI Agent Integration

An LLM agent driving this CLI should start by reading the built-in manual:

```bash
samsung-artmode agent-readme --json
```

### Stable Exit Codes

| Code | Name | Description |
|:---:|:---|:---|
| `0` | `ok` | Command completed successfully |
| `1` | `error` | Unclassified runtime error |
| `2` | `network` | TV unreachable, connection timeout, or dropped socket |
| `3` | `auth_required` | Connection denied or invalid token (remediation: `pair`) |
| `4` | `not_found` | Resource or content ID not found on TV |
| `5` | `rate_limited` | Back off before retrying |
| `6` | `invalid_input` | Parameter validation failure (e.g. brightness > 10) |
| `7` | `no_account` | No TV specified via `--account` or `--host` |

---

## Origins & Lineage

The original prototype was created by Niels Bosma as the .NET global tool [`SamsungArtMode.Console`](https://github.com/nielsbosma/SamsungArtMode.Console). SpaceCorps ported the codebase to native Rust (Edition 2024) to deliver instant cold starts, zero runtime dependencies, native OS keystore credential storage, and an autonomous agent discovery interface.

---

## License

Released under the [MIT License](LICENSE).
Copyright © 2026 SpaceCorps.
