# eero-api

Async Rust client library for the eero WiFi router API.

This crate provides a typed, async client for interacting with eero mesh WiFi routers. It covers:

- Authentication (login/verify flow)
- Network management and settings
- eero node control (LED, nightlight, reboot)
- Connected device tracking and management
- Profiles (parental controls)
- Port forwarding rules
- Activity monitoring (requires eero Plus)
- Diagnostics and speed tests
- Guest network management

## Quick Start

```rust
use eero_api::{EeroClient, InMemoryStore, ClientBuilder};

#[tokio::main]
async fn main() -> eero_api::Result<()> {
    // Create a client with an in-memory credential store
    let store = InMemoryStore::new();
    let client = ClientBuilder::new()
        .credential_store(Box::new(store))
        .build()?;

    // Authenticate with a known session token
    client.credentials().set_session_token("your_token").await?;

    // List networks on the account
    let account = client.get_account().await?;
    for net in &account.networks.data {
        println!("{}: {}", net.url, net.name.as_deref().unwrap_or("unnamed"));
    }

    // Get devices on the first network
    let network_id = 12345;
    let devices = client.get_devices(network_id).await?;
    for device in &devices {
        println!("{}", device.display_name.as_deref().unwrap_or("unknown"));
    }

    Ok(())
}
```

## Authentication

The eero API uses a two-step verification flow:

```rust
// Step 1: Send verification code to email or phone
client.login("you@example.com").await?;

// Step 2: Verify with the code (stores session token automatically)
client.verify("123456").await?;

// Now authenticated — session token is persisted in the credential store
let account = client.get_account().await?;
```

## Credential Backends

Session tokens are stored via the `CredentialStore` trait. Multiple backends are available:

| Backend | Feature Flag | Description |
|---------|-------------|-------------|
| `FileStore` | *(default)* | Plain text files in `~/.config/eero/` |
| `InMemoryStore` | *(default)* | Non-persistent, for testing or passing tokens directly |
| `KeyringStore` | `keyring` | OS keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager) |
| `OpStore` | `op` | 1Password CLI (`op`) |
| `DpapiStore` | `dpapi` | Windows DPAPI encrypted files (Windows only) |

Enable optional backends in your `Cargo.toml`:

```toml
[dependencies]
eero-api = { version = "0.1", features = ["keyring"] }
```

## API Methods

### Account

| Method | Returns | Description |
|--------|---------|-------------|
| `get_account()` | `Account` | Account info and network list |

### Networks

| Method | Returns | Description |
|--------|---------|-------------|
| `get_network(id)` | `Network` | Network details by ID |
| `get_network_by_url(url)` | `Network` | Network details by resource URL |
| `update_network(id, update)` | `Network` | Update network settings |
| `reboot_network(id)` | `Value` | Reboot all nodes |

### Speed Tests

| Method | Returns | Description |
|--------|---------|-------------|
| `run_speed_test(id)` | `Value` | Start a speed test |
| `get_speed_test(id)` | `Vec<SpeedTest>` | Latest speed test results |

### Guest Network

| Method | Returns | Description |
|--------|---------|-------------|
| `get_guest_network(id)` | `GuestNetwork` | Guest network settings |
| `update_guest_network(id, update)` | `GuestNetwork` | Update guest network |

### eero Nodes

| Method | Returns | Description |
|--------|---------|-------------|
| `get_eeros(network_id)` | `Vec<EeroNode>` | List eero nodes |
| `get_eero(url)` | `EeroNode` | Get node by resource URL |
| `update_eero(url, update)` | `EeroNode` | Update node (location, LED, nightlight) |
| `reboot_eero(url)` | `Value` | Reboot a specific node |

### Devices

| Method | Returns | Description |
|--------|---------|-------------|
| `get_devices(network_id)` | `Vec<Device>` | List connected devices |
| `get_device(url)` | `Device` | Get device by resource URL |
| `update_device(url, update)` | `Device` | Update device (nickname, block, pause, profile) |

### Profiles

| Method | Returns | Description |
|--------|---------|-------------|
| `get_profiles(network_id)` | `Vec<Profile>` | List profiles |
| `get_profile(url)` | `Profile` | Get profile by resource URL |
| `update_profile(url, update)` | `Profile` | Update profile settings |

### Port Forwarding

| Method | Returns | Description |
|--------|---------|-------------|
| `get_port_forwards(network_id)` | `Vec<PortForward>` | List port forwards |
| `create_port_forward(network_id, forward)` | `PortForward` | Create a forward |
| `delete_port_forward(url)` | `Value` | Delete a forward |

### Activity

| Method | Returns | Description |
|--------|---------|-------------|
| `get_activity(network_id)` | `ActivitySummary` | Network activity (eero Plus) |
| `get_client_activity(device_url)` | `ClientActivity` | Per-device activity |

### Diagnostics

| Method | Returns | Description |
|--------|---------|-------------|
| `get_diagnostics(network_id)` | `DiagnosticReport` | Latest diagnostic report |
| `run_diagnostics(network_id)` | `DiagnosticReport` | Run a new diagnostic check |

## Type System

All API response types are in the `eero_api::types` module. Types use `#[serde(flatten)]` with a `HashMap<String, Value>` extra field for forward compatibility — unknown fields from the API are preserved rather than silently dropped.

Update types (e.g., `NetworkUpdate`, `EeroUpdate`, `DeviceUpdate`) use `Option` fields with `#[serde(skip_serializing_if = "Option::is_none")]` so only explicitly set fields are sent to the API.

## Error Handling

All API methods return `eero_api::Result<T>`, which uses the `eero_api::Error` enum:

```rust
use eero_api::Error;

match client.get_account().await {
    Ok(account) => println!("{}", account.email.display_name),
    Err(Error::NotAuthenticated) => eprintln!("Please log in first"),
    Err(Error::Api { code, message }) => eprintln!("API error {code}: {message}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

## License

MIT — see [LICENSE](LICENSE).
