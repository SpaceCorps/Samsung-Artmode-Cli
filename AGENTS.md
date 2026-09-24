# AGENTS.md

Notes for developers and autonomous agents extending `samsung-artmode`.

`samsung-artmode` is a native Rust CLI for controlling Samsung The Frame TV Art Mode over local network WebSocket and TCP sockets. It was ported from Niels Bosma's original C# prototype (`SamsungArtMode.Console`) and upgraded to the SpaceCorps CLI Standard.

For the manual an *agent* reads at runtime, run `samsung-artmode agent-readme [--json]`. That text lives in `src/readme.rs` and is the tool's machine interface. This file is for developers and agents modifying or maintaining the source code.

---

## Development & Verification Commands

```bash
cargo build --release              # target/release/samsung-artmode
cargo test                         # unit tests + tests/cli.rs against in-process mock
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install binary on PATH
```

### Isolated Testing Environment

Always test with an isolated config directory so real keystore accounts are not modified:

```bash
export SAMSUNG_CONFIG_DIR=$(mktemp -d) SAMSUNG_SECRET_STORE=plaintext SAMSUNG_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Description |
|:---|:---|
| `SAMSUNG_CONFIG_DIR` | Overrides the configuration file directory |
| `SAMSUNG_SECRET_STORE` | Forces a keystore backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `SAMSUNG_ALLOW_PLAINTEXT_STORE=1` | Permits fallback to 0600 plaintext file where no keystore exists |
| `SAMSUNG_TV_HOST` | Fallback TV IP address if `--host` and `--account` are omitted |
| `SAMSUNG_TV_TOKEN` | Fallback auth token if `--token` is omitted |

---

## Architecture & Code Layout

```
src/
  main.rs          arg parsing, --json pre-scan, clap errors -> invalid_input envelopes
  cli.rs           clap derive hierarchy, command arguments, and docstring help text
  client.rs        Samsung TV Art Mode WebSocket/TCP protocol implementation (blocking)
  discovery.rs     SSDP multicast discovery on 239.255.255.250:1900
  commands/
    mod.rs         command dispatcher
    accounts.rs    accounts add | list | test | remove
    art.rs         pair, status, list, current, select, upload, delete, favorite, brightness, matte, slideshow
  error.rs         ErrorCode enum (= exit code) and structured Error { message, detail, remediation }
  output.rs        YAML default (serde_norway), JSON (--json), write_error envelope, obj! macro
  account.rs       multi-TV device resolution (--account, --host, env vars)
  config.rs        config.yaml, paths, atomic writes, 0600 permissions, cross-process lock
  secrets.rs       macOS Keychain (/usr/bin/security), Linux Secret Service (secret-tool), Windows DPAPI, plaintext
  readme.rs        agent-readme text and machine rules
tests/
  cli.rs           in-process mock WebSocket & TCP TV test suite
```

---

## Architectural Decisions

1. **Blocking Sockets over Tokio:**
   A CLI execution performs only 1 to 3 sequential network operations. Introducing `tokio` would add hundreds of kilobytes of runtime overhead and increase startup latency. Using blocking `TcpStream`, `rustls::StreamOwned`, and `tungstenite` gives 1–3 ms cold-start times.

2. **Self-Signed TLS Handling:**
   Samsung The Frame TVs generate self-signed or private CA TLS certificates for port 8002 (WSS) and secure binary uploads. `src/client.rs` configures `rustls` with a custom certificate verifier (`AcceptAnyCert`) to allow trusted local network communication without invalid certificate rejections.

3. **Two-Phase Binary Image Upload:**
   Uploading an image requires:
   - Requesting `send_image` over WebSocket.
   - Waiting for `ready_to_use` event containing a dedicated TCP port (`conn_info.port`), security flag, and transfer key.
   - Connecting directly via TCP to transmit a 4-byte big-endian header length, ASCII JSON metadata header, and 64KB image chunks.
   - Waiting on the original WebSocket for `image_added` containing the assigned `content_id`.

4. **Native Keystore Delegation:**
   Secrets never touch `config.yaml`. Auth tokens are stored in the host operating system's native keychain. If no keystore is present, the CLI refuses to write plaintext tokens unless explicitly opted in with `SAMSUNG_ALLOW_PLAINTEXT_STORE=1`.

5. **Multi-TV Scoping Safety:**
   Commands require explicit `--account <name>` or `--host <ip>`. If neither is provided and no environment variable is set, the CLI exits with `no_account` (`code: 7`) and lists all configured TVs to prevent accidental commands sent to the wrong room.

---

## Adding or Extending Commands

1. Add the command variant to `src/cli.rs` with doc comments.
2. Implement the API request in `src/client.rs` and add the command function in `src/commands/art.rs`.
3. Wire the dispatcher in `src/commands/mod.rs`.
4. Document the new command in `src/readme.rs`, `README.md`, `docs/llms.txt`, and `docs/llms-full.txt`.
5. Add an offline test case in `tests/cli.rs` verifying mock behavior and output assertions.
