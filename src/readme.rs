//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as structured data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "samsung-artmode",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call arguments",
                "7" => "no_account - pass --account or --host, or run samsung-artmode accounts list",
            },
        });
        return;
    }
    println!("{README}");
}

pub const API_VERSION: &str = "1.0.0";

const RULES: &[&str] = &[
    "Specify the target TV with --account <name> or --host <ip>. There is no implicit fallback.",
    "Run 'samsung-artmode accounts list' first if you do not know which TVs are configured.",
    "On code auth_required, stop and surface the remediation string. Do not retry without pairing.",
    "Image uploads require PNG or JPEG files. Pass --select to display immediately after upload.",
    "Matte IDs can be inspected with 'samsung-artmode matte list'.",
    "Brightness values must be between 0 and 10.",
    "Use --json when parsing output with jq or in an agent tool loop.",
];

const README: &str = r#"# samsung-artmode - agent operating manual

A native Rust CLI for controlling Samsung The Frame TV Art Mode over the local network.
Manage artwork, upload images, set mattes, adjust brightness, control slideshows, and pair
devices securely. Results are YAML on stdout, errors are structured on stderr, and `--json`
switches both to machine-readable JSON.

## Target Device Resolution

Pass `--account <name>` (short `-a`) or `--host <ip>` on every command.
There is no implicit fallback or guessing when multiple TVs are present.

    samsung-artmode accounts list                      # what TVs are configured
    samsung-artmode status -a living-room              # check Art Mode status
    samsung-artmode status --host 192.168.1.100        # target by IP directly

If you do not know which TV to use, run `samsung-artmode accounts list` and ask the user.

### Pairing with a New TV

    samsung-artmode pair [host] [--name <name>]

1. If no host is provided, auto-discovers Samsung TVs on the local network via SSDP.
2. Prompts the TV user to press 'Allow' on the TV screen.
3. Obtains an authentication token from the TV.
4. If `--name <name>` is provided, saves the device configuration to `config.yaml` and stores the token in the OS Keystore (Keychain, DPAPI, libsecret).

## Commands Summary

    samsung-artmode pair [host] [--name <name>]
    samsung-artmode status [-a <account> | --host <ip>]
    samsung-artmode list [-a <account> | --host <ip>] [--category <category>]
    samsung-artmode current [-a <account> | --host <ip>]
    samsung-artmode select <content-id> [--show] [-a <account> | --host <ip>]
    samsung-artmode upload <file> [--matte <matte-id>] [--select] [-a <account> | --host <ip>]
    samsung-artmode delete <content-id> [-a <account> | --host <ip>]
    samsung-artmode favorite <content-id> [--remove] [-a <account> | --host <ip>]
    samsung-artmode brightness get [-a <account> | --host <ip>]
    samsung-artmode brightness set <0-10> [-a <account> | --host <ip>]
    samsung-artmode matte list [-a <account> | --host <ip>]
    samsung-artmode matte set <content-id> <matte-id> [-a <account> | --host <ip>]
    samsung-artmode slideshow [--off] [--interval <minutes>] [--category <cat>] [--shuffle]
    samsung-artmode accounts list
    samsung-artmode accounts add <name> --host <ip> [--token <token>] [--mac <mac>] [--model <model>]
    samsung-artmode accounts test <name>
    samsung-artmode accounts remove <name> [--yes]
    samsung-artmode agent-readme [--json]

## Categories

- `MY-C0002`: My Photos (uploaded user photos)
- `MY-C0004`: Favorites
- `MY-C0008`: Samsung Art Store

## Exit Codes

    0  ok
    1  error          unclassified error - report and stop
    2  network        TV unreachable or connection dropped - verify power/network
    3  auth_required  TV rejected connection or invalid token - pair device again
    4  not_found      content ID or file not found - do not retry
    5  rate_limited   back off before trying again
    6  invalid_input  fix argument values (e.g. brightness outside 0-10)
    7  no_account     specify --account <name> or --host <ip>
"#;
