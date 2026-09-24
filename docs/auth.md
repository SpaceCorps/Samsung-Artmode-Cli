---
title: "Authentication & Pairing Guide"
description: "Pairing protocols, TV authorization, token storage, and error remediation for Samsung Art Mode CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication & Pairing Guide for Samsung Art Mode CLI

This guide describes how to pair Samsung The Frame TVs with the `samsung-artmode` CLI, how auth tokens are issued and stored, and how to troubleshoot connection errors.

## Overview

Samsung Smart TVs running Tizen use WebSocket channels for remote control and Art Mode management:
1. **Initial Pairing (Port 8001, Plain WebSocket)**: The client connects without a token to `ws://<ip>:8001/api/v2/channels/com.samsung.art-app?name=U2Ftc3VuZ0FydENsaQ==`.
2. **On-Screen Confirmation**: The TV prompts the user on-screen: *"Allow SamsungArtCli to connect?"*.
3. **Token Issuance**: When the user presses **Allow**, the TV issues a token inside the WebSocket connection response (`data.token`).
4. **Authenticated Art Mode (Port 8002, TLS WebSocket)**: All subsequent Art Mode commands connect to `wss://<ip>:8002/api/v2/channels/com.samsung.art-app?name=U2Ftc3VuZ0FydENsaQ==&token=<token>`.

## Step-by-Step Pairing Flow

### 1. Auto-Discovery & Pairing

Ensure your TV is powered on and connected to the same local Wi-Fi or Ethernet network.

```bash
# Auto-discover TV on local network and prompt to pair
samsung-artmode pair --name living-room
```

Or target the TV directly by IP:
```bash
samsung-artmode pair 192.168.1.100 --name living-room
```

### 2. Save Device to Keystore

When `--name <name>` is provided, the CLI securely stores the authentication token in your operating system's native keystore:
- **macOS**: macOS Keychain (`security add-generic-password`)
- **Windows**: Windows DPAPI (`CryptProtectData`)
- **Linux**: FreeDesktop Secret Service daemon (`secret-tool`)

Device metadata (IP host, model, MAC) is written to `~/.config/samsung-artmode-cli/config.yaml` with file permissions restricted to `0600`.

### 3. Verification

Test the stored account:
```bash
samsung-artmode accounts test living-room
```

## Environment Variables & Direct Flags

In headless environments or scripting pipelines, credentials can be passed directly:
- `--account <name>` (short `-a <name>`): Selects a configured TV from keystore/config.
- `--host <ip>`: Overrides target IP address.
- `--token <token>`: Overrides authentication token.
- `SAMSUNG_TV_HOST`: Environment variable fallback for host IP.
- `SAMSUNG_TV_TOKEN`: Environment variable fallback for token.

## Troubleshooting & Remediation

| Exit Code | Machine Code | Cause | Remediation |
| :--- | :--- | :--- | :--- |
| `2` | `network` | TV is off, wrong IP, or firewall blocked port 8001/8002 | Check TV power and IP; ensure client is on same subnet |
| `3` | `auth_required` | TV rejected connection or token was invalidated | Re-run `samsung-artmode pair <ip>` and press 'Allow' on TV |
| `7` | `no_account` | No TV specified via `--account` or `--host` | Run `samsung-artmode accounts list` or `samsung-artmode pair` |
