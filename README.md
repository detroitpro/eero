# eeroctl

> **This is heka's maintained fork of [erikh/eero](https://github.com/erikh/eero).** It adds
> fixes and features for running eeroctl from automation: the session token stays out of logs,
> output and world-readable files; device updates take effect; DHCP reservations; a `--yes` guard
> on destructive commands; and release binaries. See [docs/plan.md](docs/plan.md) for what and
> why. Issues and pull requests go to [heka-vault/eero](https://github.com/heka-vault/eero/issues).

A command-line tool for managing [eero](https://eero.com) WiFi mesh routers. Built on the unofficial eero API, eeroctl gives you full control over your network from the terminal: view connected devices, manage profiles, configure port forwards, run speed tests, and more.

This project consists of two crates:

- **eeroctl** — The CLI application (AGPL-3.0)
- **eero-api** — A standalone async Rust client library for the eero API (MIT) — see [eero-api/README.md](eero-api/README.md)

## Things You Can Do With It

See all the devices on your network at a glance:

```
$ eeroctl device list
ID           NAME
04421a06d967 islay-49
ae94f29cf6d1 speyside
b0416f0b2f70 town-os
666ce339fe60 66:6c:e3:39:fe:60
72dcf28ae564 Watch
```

Set up port forwarding with a single command:

```
$ eeroctl forward create --help
Create a port forward

Usage: eeroctl forward create [OPTIONS] --ip <IP> --port <PORT> [NETWORK]

Arguments:
  [NETWORK]  Network name (SSID) or `id:XXXXX`. Defaults to first network

Options:
      --ip <IP>                        IP address of the device (usually one with a DHCP reservation)
      --port <PORT>                    Port on the device
      --external-port <EXTERNAL_PORT>  Port on the WAN side (defaults to --port)
      --protocol <PROTOCOL>            [default: tcp] [possible values: tcp, udp, both]
      --description <DESCRIPTION>      Label shown in the eero app
  -h, --help                           Print help
```

## Testing and Compatibility

eeroctl is developed and tested against an **eero 6** router. The eero API is undocumented and not officially supported by eero, so behavior may vary across hardware generations and firmware versions.

If you have a different eero model and would like to help verify compatibility:

1. Install eeroctl and authenticate (see below)
2. Run through the basic commands (`account get`, `network get`, `eero list`, `device list`)
3. Report any issues or unexpected output at [github.com/heka-vault/eero/issues](https://github.com/heka-vault/eero/issues). Redact network IDs, MAC addresses, IPs and tokens first.

The test suite includes unit tests, integration tests with a mock API server, and optional live tests that run against a real eero device. See [Running Tests](#running-tests) for details.

## Installation

Linux x86_64 binaries (glibc 2.35+, OpenSSL 3, so Ubuntu 22.04 or newer) are attached to each
[release](https://github.com/heka-vault/eero/releases) with a `.sha256`:

```bash
v=v0.2.2
curl -fsSLO "https://github.com/heka-vault/eero/releases/download/$v/eeroctl-$v-x86_64-linux-gnu.tar.gz"
curl -fsSL "https://github.com/heka-vault/eero/releases/download/$v/eeroctl-$v-x86_64-linux-gnu.tar.gz.sha256" | sha256sum -c
tar xzf "eeroctl-$v-x86_64-linux-gnu.tar.gz" && install -m 755 "eeroctl-$v-x86_64-linux-gnu/eeroctl" ~/.local/bin/
```

Building from source requires [Rust](https://www.rust-lang.org/tools/install) 1.85+ (edition 2024).

```bash
# From source
cargo install --path eeroctl

# Or build without installing
cargo build --release -p eeroctl
```

## Authentication

eero uses a two-step verification flow. You authenticate once and the session token is stored locally in `~/.config/eero/`.

```bash
# Step 1: Request a verification code (sent via SMS or email)
eeroctl auth login you@example.com

# Step 2: Enter the code you received
eeroctl auth verify 123456
```

After verification you're authenticated and can run any command.

To skip the interactive flow, set the `EERO_SESSION_TOKEN` environment variable or pass `--user-token`:

```bash
export EERO_SESSION_TOKEN="your_token_here"
eeroctl account get
```

To log out and clear stored credentials:

```bash
eeroctl auth logout
```

## Quick Start

```bash
# See your account and networks
eeroctl account get

# List devices on your network
eeroctl device list

# Get details for a specific device
eeroctl device get "MacBook Pro"

# Run a speed test, then fetch the results
eeroctl network speed-test run
eeroctl network speed-test get

# Check eero node status
eeroctl eero list
```

## Example Output

All output is YAML by default (with syntax highlighting in a terminal). Use `--json` for JSON or `--jq` for filtered output.

### account get

```
$ eeroctl account get
networks:
  count: 1
  data:
  - url: /2.2/networks/12345
    name: MyNetwork
premium_status: active
is_premium: true
role: owner
```

### network get

```
$ eeroctl network get
name: MyNetwork
health:
  internet:
    status: connected
  eero_network:
    status: connected
speed:
  down:
    value: 500.0
    units: Mbps
  up:
    value: 50.0
    units: Mbps
clients:
  count: 10
eeros:
  count: 3
dns:
  provider: cloudflare
  caching: true
sqm: true
band_steering: true
upnp: true
ipv6: true
wpa3: true
target_firmware: 7.0.0
```

### device list

```
$ eeroctl device list
ID           NAME
2001         MacBook Pro
2002         galaxy-phone
2003         Living Room TV
```

### device get

```
$ eeroctl device get "MacBook Pro"
mac: 11:22:33:44:55:66
manufacturer: Apple
ip: 192.168.4.100
connected: true
wireless: true
connection_type: wireless
source:
  location: Living Room
profile:
  name: Default
paused: false
blocked: false
frequency: 5 GHz
auth: wpa3
```

### eero list

```
$ eeroctl eero list
- model: eero Pro 6E
  location: Living Room
  status: green
  connected_clients_count: 5
  gateway: true
  ip_address: 192.168.4.1
  mesh_quality_bars: 5
  wired: true
- model: eero 6+
  location: Bedroom
  status: green
  connected_clients_count: 3
  gateway: false
  ip_address: 192.168.4.2
  mesh_quality_bars: 4
  wired: false
```

### eero get

```
$ eeroctl eero get "Living Room"
model: eero Pro 6E
location: Living Room
os_version: 7.0.0-123
status: green
status_light:
  enabled: true
  brightness: 100
connected_clients_count: 5
nightlight:
  enabled: false
  schedule:
    'on': '21:00'
    'off': '07:00'
  brightness: 50
gateway: true
ip_address: 192.168.4.1
mesh_quality_bars: 5
wired: true
update_available: false
```

### speed-test run / get

```
$ eeroctl network speed-test run
Speed test started. Please run `speed-test get` in a moment.

$ eeroctl network speed-test get
- date: '2024-06-15T12:00:00Z'
  up_mbps: 48.5
  down_mbps: 495.2
```

### profile list / get

```
$ eeroctl profile list
- name: Default
  paused: false
- name: Kids
  paused: false

$ eeroctl profile get "Kids"
name: Kids
paused: false
blocked_applications:
- tiktok
dns_policies:
  block:
  - adult
  safe_search_enabled: true
  youtube_restricted: true
schedule:
  enabled: true
  time_ranges: []
```

### forward list

```
$ eeroctl forward list
- url: /2.2/networks/12345/forwards/4001
  enabled: true
  ip: 192.168.4.50
  client_port: 8080
  gateway_port: 80
  protocol: tcp
  description: Web Server
  reservation: /2.2/networks/12345/reservations/5001
```

### guest get

```
$ eeroctl network guest get
name: MyNetwork Guest
password: guestpass
enabled: true
```

### activity get

```
$ eeroctl activity get
insights:
- type: top_domain
  category: streaming
  count: 1500
  domain: netflix.com
- type: top_domain
  category: social
  count: 800
  domain: twitter.com
counts:
  blocked: 42
  allowed: 15000
  inspected: 15042
```

### diagnostics get

```
$ eeroctl diagnostics get
status: completed
results:
- type: internet
  status: pass
  details:
    latency_ms: 12
- type: dns
  status: pass
  details:
    resolver: 1.1.1.1
```

### jq filter

```
$ eeroctl --jq '.name' network get
"MyNetwork"

$ eeroctl --jq '.[].display_name' device list -m
"MacBook Pro"
"galaxy-phone"
"Living Room TV"
```

## Network Resolution

Most commands accept an optional network argument. If omitted, the first network on your account is used automatically:

```bash
# Uses your default (first) network
eeroctl device list

# Specify a network by SSID
eeroctl device list "My Home WiFi"

# Use a numeric network ID with the id: prefix
eeroctl network get "id:12345"
```

Run `eeroctl account get` to see your available networks and their names.

## Name Resolution

eeros, devices, and profiles can be referenced by human-readable names instead of numeric IDs:

```bash
# eero by location name
eeroctl eero get "Living Room"

# Device by display name, nickname, or hostname
eeroctl device get "MacBook Pro"

# Profile by name
eeroctl profile get "Kids"
```

If a name matches multiple objects, an error is returned listing the matches. To use a numeric ID directly, prefix it with `id:`:

```bash
eeroctl eero get "id:1001"
eeroctl device get "id:2001"
```

## Commands

### Authentication

| Command | Description |
|---|---|
| `auth login <email_or_phone>` | Begin login (sends verification code) |
| `auth verify <code>` | Complete login with verification code |
| `auth logout` | Log out and clear stored credentials |

### Account

| Command | Description |
|---|---|
| `account get` | Show account information and network list |

### Network

| Command | Description |
|---|---|
| `network get [network]` | Show network details (name, health, DNS, clients) |
| `network set [network] --flag value` | Update network settings |
| `network reboot [network]` | Reboot all eero nodes on the network |
| `network data-usage [network] --start T --end T` | Upload/download totals for a window (`--cadence`, `--timezone`, `--by devices\|eeros`) |

**Network set flags:** `--name`, `--password`, `--sqm`, `--band-steering`, `--thread`, `--upnp`, `--ipv6`, `--wpa3`

### Speed Test

| Command | Description |
|---|---|
| `network speed-test run [network]` | Run a new speed test |
| `network speed-test get [network]` | Get the latest speed test result |

### Guest Network

| Command | Description |
|---|---|
| `network guest get [network]` | Show guest network settings |
| `network guest set [network] --flag value` | Update guest network settings |

**Guest set flags:** `--name`, `--password`, `--enabled`

### eero Nodes

| Command | Description |
|---|---|
| `eero list [network]` | List all eero nodes on the network |
| `eero get <eero> [--network N]` | Show details for an eero node (by location name or `id:XXXXX`) |
| `eero set <eero> [--network N] --flag value` | Update eero node settings |
| `eero reboot <eero> [--network N]` | Reboot a specific eero node |

**eero set flags:** `--location`, `--led-on`, `--nightlight-enabled`, `--nightlight-brightness`, `--nightlight-on`, `--nightlight-off`

### Devices

| Command | Description |
|---|---|
| `device list [network]` | List connected devices (human-readable table) |
| `device list [network] -m` | List devices in structured (JSON/YAML) output |
| `device get <device> [--network N]` | Show full details for a device (by name or `id:XXXXX`) |
| `device set <device> [--network N] --flag value` | Update device settings |

**Device set flags:** `--nickname`, `--blocked`, `--paused`, `--profile`

### Profiles

| Command | Description |
|---|---|
| `profile list [network]` | List profiles (parental controls) |
| `profile get <profile> [--network N]` | Show profile details (by name or `id:XXXXX`) |
| `profile set <profile> [--network N] --flag value` | Update profile settings |

**Profile set flags:** `--name`, `--paused`

### Port Forwarding

| Command | Description |
|---|---|
| `forward list [network]` | List port forwarding rules |
| `forward create [network] --ip IP --port PORT` | Create a port forward |
| `forward delete <id> [--network N]` | Delete a port forward by the ID `forward list` shows |

**Forward create flags:** `--ip` (required), `--port` (required, the device's port), `--external-port` (WAN side, defaults to `--port`), `--protocol` (`tcp`, `udp` or `both`; default `tcp`), `--description`

### DHCP Reservations

| Command | Description |
|---|---|
| `reservation list [network]` | List DHCP reservations |
| `reservation create [network] --ip IP --mac MAC [--description D]` | Reserve an IP address for a MAC address |
| `reservation delete <mac\|ip\|id:XXXXX> [--network N]` | Delete a reservation, found by MAC, IP or ID |

### Activity

| Command | Description |
|---|---|
| `activity get [network]` | Get network activity summary (requires eero Plus) |
| `activity get-client <device> [--network N]` | Get activity for a specific device |

### Diagnostics

| Command | Description |
|---|---|
| `diagnostics get [network]` | Get the latest diagnostic report |
| `diagnostics run [network] [--symptom S] [--device D]` | Run a new diagnostic check (newer firmware requires `--symptom`) |

## Output Formats

By default, output is syntax-highlighted YAML. You can change this:

```bash
# JSON output
eeroctl --json device list -m

# Disable syntax highlighting (useful for piping)
eeroctl -n device list -m

# Apply a jq filter (output is always JSON, one value per line)
eeroctl --jq '.name' network get
eeroctl --jq '.[].display_name' device list -m
```

## Global Options

| Flag | Env Var | Description |
|---|---|---|
| `--json` | | Output JSON instead of YAML |
| `--jq <EXPR>` | | Apply a jq filter expression (always outputs JSON) |
| `-n` / `--no-color` | | Disable syntax highlighting |
| `--user-token <TOKEN>` | `EERO_SESSION_TOKEN` | Session token (skips login flow) |
| `--log-level <LEVEL>` | `LOG_LEVEL` | Logging level (debug, info, warn, error, trace). Tokens, passwords and account contact details are masked |
| `-y` / `--yes` | | Skip the confirmation for reboots, deletes, Wi-Fi password changes and `auth logout`. Without a terminal these refuse to run unless `--yes` is given |
| `--version` | | Print the version |

## Running Tests

```bash
# Run linting and all tests (skips live tests)
make test

# Run only eero-api tests
cargo test -p eero-api -- --skip live_

# Run only eeroctl tests
cargo test -p eeroctl
```

### Live Tests

Live tests run against a real eero router on your network. They require authentication and the router's IP address:

```bash
# Specify the eero IP address
make test-live EERO_IP=192.168.4.1

# Auto-detect from default gateway
make test-live-default
```

Live tests perform read-only operations and will not change your network configuration.

## License

- **eeroctl** (CLI) — [AGPL-3.0-or-later](LICENSE)
- **eero-api** (library) — [MIT](eero-api/LICENSE)
