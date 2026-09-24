---
title: "Samsung Art Mode CLI"
description: "A fast native command-line tool and agent interface for Samsung The Frame TV Art Mode. Built in Rust for developers, home automation, and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/Samsung-Artmode-Cli/index.md"
---

# Samsung Art Mode CLI

A blazing fast native command-line tool and agent interface for Samsung The Frame TV Art Mode. Control artwork, upload images, adjust brightness, change frame mattes, and automate slideshows over your local network.

## Quickstart

```bash
# Pair with TV on local network (auto-discovery or direct IP)
samsung-artmode pair --name living-room

# Get Art Mode status
samsung-artmode status -a living-room

# Upload a photograph and display it immediately with a polar matte
samsung-artmode upload family-portrait.jpg --matte flexible_polar --select -a living-room

# Configure a 30-minute shuffled slideshow of your personal photos
samsung-artmode slideshow --interval 30 --shuffle -a living-room
```

## Features

- **Native Speed & Zero Dependencies**: Standalone Rust binary with 1–3 ms cold start times.
- **AI Agent Native**: Structured JSON output (`--json`), rich error envelopes, and built-in `agent-readme`.
- **Operating System Keystore**: TV authentication tokens stored securely in macOS Keychain, Windows DPAPI, or Linux Secret Service.
- **Deterministic Multi-TV Management**: Explicit `--account <name>` scoping prevents accidental mutations across multi-TV setups.
- **Local Network Privacy**: Operates 100% locally via WebSocket and TCP binary sockets with zero third-party cloud reliance.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/Samsung-Artmode-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/Samsung-Artmode-Cli/llms-full.txt)
- [Authentication Guide](https://spacecorps.github.io/Samsung-Artmode-Cli/auth.md)
- [Pricing](https://spacecorps.github.io/Samsung-Artmode-Cli/pricing.md)
- [GitHub Repository](https://github.com/SpaceCorps/Samsung-Artmode-Cli)
