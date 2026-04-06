# Samsung Art Mode CLI

A CLI tool for controlling Samsung The Frame TV's Art Mode over the local network. Upload images, manage artwork, control brightness, mattes, and slideshows. Built with [Spectre.Console](https://spectreconsole.net/) and .NET 10.

## Install

```bash
dotnet tool install --global SamsungArtMode.Console
```

## Pairing

First, pair with your TV (it must be powered on):

```bash
# Auto-discover on the local network
samsung-art pair

# Or specify the IP directly
samsung-art pair 192.168.1.100
```

Approve the connection on your TV when prompted. Save the returned token:

```bash
export SAMSUNG_TV_HOST="192.168.1.100"
export SAMSUNG_TV_TOKEN="your-token-here"
```

Or pass `--host` and `--token` per-command.

## Commands

```
samsung-art pair [HOST]                          Pair with TV (auto-discovers if no host given)
samsung-art status                               Get art mode status (on/off)
samsung-art list [--category]                    List images on the TV
samsung-art current                              Get currently displayed artwork
samsung-art select <CONTENT_ID> [--show]         Select an image to display
samsung-art upload <FILE> [--matte] [--select]   Upload a PNG/JPEG image
samsung-art delete <CONTENT_ID>                  Delete an image
samsung-art favorite <CONTENT_ID> [--remove]     Add/remove from favorites
samsung-art brightness get                       Get current brightness
samsung-art brightness set <VALUE>               Set brightness (0-10)
samsung-art matte list                           List available mattes
samsung-art matte set <CONTENT_ID> <MATTE_ID>   Apply a matte to an image
samsung-art slideshow [--interval] [--shuffle]   Configure slideshow
samsung-art slideshow --off                      Disable slideshow
```

## Examples

```bash
# Upload and immediately display an image
samsung-art upload photo.jpg --select

# Set up a shuffled slideshow from My Photos, rotating every 30 minutes
samsung-art slideshow --interval 30 --shuffle

# Change the matte on an image
samsung-art matte set SAM-F0206 flexible_polar

# JSON output for scripting
samsung-art list --format json
```

## Global Options

| Flag | Description |
|---|---|
| `--host <ip>` | Override `SAMSUNG_TV_HOST` env var |
| `--token <token>` | Override `SAMSUNG_TV_TOKEN` env var |
| `--format yaml\|json` | Output format (default: yaml) |
| `--verbose` | Print WebSocket messages to stderr |

## Error Handling

Errors are written to stderr as YAML with predictable exit codes:

- `0` — success
- `1` — user/input error (bad arguments, TV denied connection)
- `2` — network/connection error

## How It Works

Communicates with the TV over WebSocket (port 8001 for pairing, port 8002 with TLS for authenticated commands). Image uploads use a secondary TCP binary socket. TV discovery uses SSDP multicast.
