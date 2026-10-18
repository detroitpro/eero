use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use clap::{Parser, Subcommand};
use eero_api::types::device::{Device, DeviceUpdate};
use eero_api::types::eero::{EeroNode, EeroUpdate, NightlightScheduleUpdate, NightlightUpdate};
use eero_api::types::forward::NewPortForward;
use eero_api::types::network::{GuestNetworkUpdate, NetworkUpdate};
use eero_api::types::profile::{Profile, ProfileUpdate};
use eero_api::types::account::NetworkRef;
use eero_api::types::data_usage::DataUsageQuery;
use eero_api::types::diagnostic::DiagnosticsRequest;
use eero_api::types::reservation::{NewReservation, Reservation};
use eero_api::{ClientBuilder, EeroClient, InMemoryStore};
use jaq_core::load::{Arena, File, Loader};
use jaq_core::{Compiler, Ctx, RcIter};
use jaq_json::Val;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Parser)]
#[command(name = "eeroctl", version, about = "eero WiFi router management CLI")]
struct Cli {
    /// Skip the confirmation prompt for reboots, deletes, Wi-Fi password
    /// changes and logout. Required when there is no terminal to ask on.
    #[arg(long, short = 'y', global = true)]
    yes: bool,

    /// Session token to use instead of the login/verify flow. Prefer the
    /// environment variable: a flag is visible to other users in `ps`.
    #[arg(long, global = true, env = "EERO_SESSION_TOKEN", hide_env_values = true)]
    user_token: Option<String>,

    /// Output in JSON format instead of YAML
    #[arg(long, global = true)]
    json: bool,

    /// Apply a jq filter expression to the output (implies JSON)
    #[arg(long, global = true)]
    jq: Option<String>,

    /// Disable syntax highlighting
    #[arg(long, short = 'n', global = true)]
    no_color: bool,

    /// Enable logging at the given level (e.g. debug, info, warn, error, trace)
    #[arg(long, global = true, env = "LOG_LEVEL")]
    log_level: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Authentication commands
    #[command(subcommand)]
    Auth(AuthCmd),
    /// Account commands
    #[command(subcommand)]
    Account(AccountCmd),
    /// Network commands
    #[command(subcommand)]
    Network(NetworkCmd),
    /// Eero node commands
    #[command(subcommand)]
    Eero(EeroCmd),
    /// Device commands
    #[command(subcommand)]
    Device(DeviceCmd),
    /// Profile commands
    #[command(subcommand)]
    Profile(ProfileCmd),
    /// Port forwarding commands
    #[command(subcommand)]
    Forward(ForwardCmd),
    /// DHCP reservation commands
    #[command(subcommand)]
    Reservation(ReservationCmd),
    /// Activity commands
    #[command(subcommand)]
    Activity(ActivityCmd),
    /// Diagnostics commands
    #[command(subcommand)]
    Diagnostics(DiagnosticsCmd),
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Begin login (sends verification code)
    Login {
        /// Email address or phone number
        email_or_phone: String,
    },
    /// Verify login with code
    Verify {
        /// Verification code
        code: String,
    },
    /// Log out and clear credentials
    Logout,
}

#[derive(Subcommand)]
enum AccountCmd {
    /// Get account information
    Get,
}

#[derive(Subcommand)]
enum NetworkCmd {
    /// Get network details
    Get {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Get network by resource URL
    #[command(hide = true)]
    GetByUrl {
        /// Resource URL (e.g. /2.2/networks/12345)
        url: String,
    },
    /// Set network settings
    Set {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        sqm: Option<bool>,
        #[arg(long)]
        band_steering: Option<bool>,
        #[arg(long)]
        thread: Option<bool>,
        #[arg(long)]
        upnp: Option<bool>,
        #[arg(long)]
        ipv6: Option<bool>,
        #[arg(long)]
        wpa3: Option<bool>,
    },
    /// Reboot the network
    Reboot {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Speed test commands
    #[command(subcommand)]
    SpeedTest(SpeedTestCmd),
    /// Guest network commands
    #[command(subcommand)]
    Guest(GuestCmd),
    /// Upload and download totals for a time window
    DataUsage {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        /// Window start, ISO-8601 (e.g. 2026-08-24T04:00:00Z)
        #[arg(long)]
        start: String,
        /// Window end, ISO-8601
        #[arg(long)]
        end: String,
        /// Bucket size (e.g. hourly, daily)
        #[arg(long, default_value = "hourly")]
        cadence: String,
        /// IANA timezone the buckets align to
        #[arg(long, default_value = "UTC")]
        timezone: String,
        /// Break totals down by device or by eero node instead of a time series
        #[arg(long, value_enum)]
        by: Option<UsageBreakdown>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum UsageBreakdown {
    Devices,
    Eeros,
}

#[derive(Subcommand)]
enum SpeedTestCmd {
    /// Run a new speed test
    Run {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Get latest speed test result
    Get {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
}

#[derive(Subcommand)]
enum GuestCmd {
    /// Get guest network settings
    Get {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Set guest network settings
    Set {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        enabled: Option<bool>,
    },
}

#[derive(Subcommand)]
enum EeroCmd {
    /// List eero nodes on a network
    List {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Get a specific eero node
    Get {
        /// Eero name (location) or `id:XXXXX`
        eero: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
    },
    /// Set eero node settings
    Set {
        /// Eero name (location) or `id:XXXXX`
        eero: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
        #[arg(long)]
        location: Option<String>,
        #[arg(long)]
        led_on: Option<bool>,
        #[arg(long)]
        nightlight_enabled: Option<bool>,
        #[arg(long)]
        nightlight_brightness: Option<u32>,
        /// Nightlight schedule on time (e.g. "20:00")
        #[arg(long)]
        nightlight_on: Option<String>,
        /// Nightlight schedule off time (e.g. "06:00")
        #[arg(long)]
        nightlight_off: Option<String>,
    },
    /// Reboot an eero node
    Reboot {
        /// Eero name (location) or `id:XXXXX`
        eero: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
    },
}

#[derive(Subcommand)]
enum DeviceCmd {
    /// List devices on a network (plain text table by default)
    List {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        /// Machine-readable output (structured JSON/YAML)
        #[arg(short, long)]
        machine: bool,
    },
    /// Get full details of a device by name or ID
    Get {
        /// Device name or `id:XXXXX`
        device: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
    },
    /// Set device settings
    Set {
        /// Device name or `id:XXXXX`
        device: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
        #[arg(long)]
        nickname: Option<String>,
        #[arg(long)]
        blocked: Option<bool>,
        #[arg(long)]
        paused: Option<bool>,
        #[arg(long)]
        profile: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProfileCmd {
    /// List profiles on a network
    List {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Get a specific profile
    Get {
        /// Profile name or `id:XXXXX`
        profile: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
    },
    /// Set profile settings
    Set {
        /// Profile name or `id:XXXXX`
        profile: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        paused: Option<bool>,
    },
}

#[derive(Subcommand)]
enum ForwardCmd {
    /// List port forwards on a network
    List {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Create a port forward
    Create {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        /// Internal IP address
        #[arg(long)]
        ip_address: String,
        /// Internal port
        #[arg(long)]
        port: String,
        /// External port (defaults to same as internal)
        #[arg(long)]
        external_port: Option<String>,
        /// Protocol (tcp, udp, or tcp_udp)
        #[arg(long)]
        protocol: Option<String>,
    },
    /// Delete a port forward
    Delete {
        /// Port forward ID
        forward_id: String,
    },
}

#[derive(Subcommand)]
enum ReservationCmd {
    /// List DHCP reservations on a network
    List {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Reserve an IP address for a MAC address
    Create {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        /// IP address to reserve
        #[arg(long)]
        ip: String,
        /// MAC address of the device (aa:bb:cc:dd:ee:ff)
        #[arg(long)]
        mac: String,
        /// Label shown in the eero app
        #[arg(long)]
        description: Option<String>,
    },
    /// Delete a DHCP reservation
    Delete {
        /// Reservation MAC address, IP address, or `id:XXXXX`
        reservation: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
    },
}

/// Find a reservation by `id:XXXXX`, MAC address (any case, `:` or `-`), or IP address.
fn find_reservation<'a>(reservations: &'a [Reservation], arg: &str) -> Result<&'a Reservation> {
    let normalize_mac = |m: &str| m.to_ascii_lowercase().replace('-', ":");
    let matches: Vec<&Reservation> = if let Some(id) = arg.strip_prefix("id:") {
        reservations
            .iter()
            .filter(|r| extract_url_id(&r.url) == id)
            .collect()
    } else if arg.parse::<std::net::IpAddr>().is_ok() {
        reservations
            .iter()
            .filter(|r| r.ip.as_deref() == Some(arg))
            .collect()
    } else {
        let mac = normalize_mac(arg);
        reservations
            .iter()
            .filter(|r| r.mac.as_deref().map(normalize_mac).as_deref() == Some(mac.as_str()))
            .collect()
    };
    match matches.as_slice() {
        [one] => Ok(one),
        [] => anyhow::bail!("no reservation matches {arg:?}"),
        _ => anyhow::bail!("{} reservations match {arg:?}; use id:XXXXX", matches.len()),
    }
}

#[derive(Subcommand)]
enum ActivityCmd {
    /// Get network activity summary
    Get {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Get activity for a specific client/device
    GetClient {
        /// Device name or `id:XXXXX`
        device: String,
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        #[arg(long)]
        network: Option<String>,
    },
}

#[derive(Subcommand)]
enum DiagnosticsCmd {
    /// Get latest diagnostic report
    Get {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
    },
    /// Run a new diagnostic check
    Run {
        /// Network name (SSID) or `id:XXXXX`. Defaults to first network.
        network: Option<String>,
        /// What is wrong, as the eero app's health check asks. Newer
        /// firmware requires it.
        #[arg(long)]
        symptom: Option<String>,
        /// Device name or `id:XXXXX` to focus the check on
        #[arg(long)]
        device: Option<String>,
    },
}

fn is_default(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::Bool(b) => !b,
        serde_json::Value::Number(n) => n.as_f64() == Some(0.0),
        serde_json::Value::String(s) => s.is_empty(),
        serde_json::Value::Array(a) => a.is_empty(),
        serde_json::Value::Object(m) => m.is_empty(),
    }
}

fn strip_defaults(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let cleaned: serde_json::Map<_, _> = map
                .into_iter()
                .map(|(k, v)| (k, strip_defaults(v)))
                .filter(|(_, v)| !is_default(v))
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(strip_defaults).collect())
        }
        other => other,
    }
}

/// Ask before an action that disrupts the network or deletes configuration.
/// With `--yes` it goes ahead; without a terminal it refuses, so a script or
/// agent can't do it by accident.
fn confirm(action: &str, yes: bool) -> Result<()> {
    let interactive = io::stdin().is_terminal() && io::stderr().is_terminal();
    confirm_with(action, yes, interactive, &mut io::stdin().lock(), &mut io::stderr())
}

fn confirm_with(
    action: &str,
    yes: bool,
    interactive: bool,
    input: &mut impl io::BufRead,
    prompt: &mut impl Write,
) -> Result<()> {
    if yes {
        return Ok(());
    }
    if !interactive {
        anyhow::bail!("refusing to {action} without --yes (no terminal to confirm on)");
    }
    write!(prompt, "This will {action}. Continue? [y/N] ")?;
    prompt.flush()?;
    let mut answer = String::new();
    input.read_line(&mut answer)?;
    match answer.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(()),
        _ => anyhow::bail!("aborted"),
    }
}

struct Output {
    json: bool,
    no_color: bool,
    is_tty: bool,
    jq_filter: Option<String>,
}

impl Output {
    fn new(json: bool, no_color: bool, jq_filter: Option<String>) -> Self {
        Self {
            json,
            no_color,
            is_tty: io::stdout().is_terminal(),
            jq_filter,
        }
    }

    fn print(&self, value: &impl serde::Serialize) -> Result<()> {
        if let Some(ref expr) = self.jq_filter {
            return self.print_jq(value, expr);
        }

        let (text, lang) = if self.json {
            (serde_json::to_string_pretty(value)?, "json")
        } else {
            let cleaned = strip_defaults(serde_json::to_value(value)?);
            (serde_yaml::to_string(&cleaned)?, "yaml")
        };

        let use_color = self.is_tty && !self.no_color;
        if use_color {
            let ss = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            let syntax = ss.find_syntax_by_extension(lang).unwrap();
            let theme = &ts.themes["base16-ocean.dark"];
            let mut h = HighlightLines::new(syntax, theme);
            let mut stdout = io::stdout().lock();
            for line in LinesWithEndings::from(&text) {
                let ranges = h.highlight_line(line, &ss)?;
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                write!(stdout, "{}", escaped)?;
            }
            write!(stdout, "\x1b[0m")?;
            if !text.ends_with('\n') {
                writeln!(stdout)?;
            }
        } else {
            print!("{}", text);
            if !text.ends_with('\n') {
                println!();
            }
        }

        Ok(())
    }

    fn print_jq(&self, value: &impl serde::Serialize, expr: &str) -> Result<()> {
        let json_val = serde_json::to_value(value)?;

        let program = File {
            code: expr,
            path: (),
        };
        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();
        let modules = loader
            .load(&arena, program)
            .map_err(|errs| anyhow::anyhow!("jq parse error: {:?}", errs))?;

        let filter = Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|errs| anyhow::anyhow!("jq compile error: {:?}", errs))?;

        let inputs = RcIter::new(core::iter::empty());
        let out = filter.run((Ctx::new([], &inputs), Val::from(json_val)));

        let mut stdout = io::stdout().lock();
        for result in out {
            let val = result.map_err(|e| anyhow::anyhow!("jq error: {e}"))?;
            let json: serde_json::Value = val.into();
            writeln!(stdout, "{}", serde_json::to_string_pretty(&json)?)?;
        }
        Ok(())
    }

    fn print_ok(&self) -> Result<()> {
        self.print(&serde_json::json!({"status": "ok"}))
    }
}

#[derive(Debug, PartialEq)]
enum NetworkArg {
    Literal(u64),
    Ssid(String),
}

fn parse_network_arg(network: &str) -> Result<NetworkArg> {
    if let Some(id_str) = network.strip_prefix("id:") {
        let id = id_str
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("invalid network ID: {}", id_str))?;
        Ok(NetworkArg::Literal(id))
    } else {
        Ok(NetworkArg::Ssid(network.to_string()))
    }
}

fn extract_network_id(url: &str) -> Result<u64> {
    url.rsplit('/')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| anyhow::anyhow!("cannot extract network ID from URL: {}", url))
}

fn find_network_by_ssid(networks: &[NetworkRef], ssid: &str) -> Result<u64> {
    let matches: Vec<_> = networks
        .iter()
        .filter(|n| n.name.as_deref() == Some(ssid))
        .collect();

    match matches.len() {
        0 => Err(anyhow::anyhow!("no network found with name {:?}", ssid)),
        1 => extract_network_id(&matches[0].url),
        n => Err(anyhow::anyhow!(
            "{} networks found with name {:?}; use `id:XXXXX` to specify",
            n,
            ssid
        )),
    }
}

async fn resolve_network_id(client: &EeroClient, network: &str) -> Result<u64> {
    match parse_network_arg(network)? {
        NetworkArg::Literal(id) => Ok(id),
        NetworkArg::Ssid(ssid) => {
            let account = client.get_account().await?;
            find_network_by_ssid(&account.networks.data, &ssid)
        }
    }
}

async fn resolve_network_id_or_default(
    client: &EeroClient,
    network: Option<&str>,
) -> Result<u64> {
    match network {
        Some(n) => resolve_network_id(client, n).await,
        None => {
            let account = client.get_account().await?;
            let first = account
                .networks
                .data
                .first()
                .ok_or_else(|| anyhow::anyhow!("no networks found on this account"))?;
            extract_network_id(&first.url)
        }
    }
}

fn require_network_set_flags(update: &NetworkUpdate) -> Result<()> {
    if update.name.is_none()
        && update.password.is_none()
        && update.sqm.is_none()
        && update.band_steering.is_none()
        && update.thread.is_none()
        && update.upnp.is_none()
        && update.ipv6.is_none()
        && update.wpa3.is_none()
    {
        anyhow::bail!("no settings specified; use --name, --password, --sqm, etc.");
    }
    Ok(())
}

fn require_guest_set_flags(update: &GuestNetworkUpdate) -> Result<()> {
    if update.name.is_none() && update.password.is_none() && update.enabled.is_none() {
        anyhow::bail!("no settings specified; use --name, --password, --enabled");
    }
    Ok(())
}

fn require_eero_set_flags(update: &EeroUpdate) -> Result<()> {
    if update.location.is_none() && update.led_on.is_none() && update.nightlight.is_none() {
        anyhow::bail!(
            "no settings specified; use --location, --led-on, --nightlight-enabled, etc."
        );
    }
    Ok(())
}

fn require_device_set_flags(update: &DeviceUpdate) -> Result<()> {
    if update.nickname.is_none()
        && update.blocked.is_none()
        && update.paused.is_none()
        && update.profile.is_none()
    {
        anyhow::bail!("no settings specified; use --nickname, --blocked, --paused, --profile");
    }
    Ok(())
}

fn require_profile_set_flags(update: &ProfileUpdate) -> Result<()> {
    if update.name.is_none() && update.paused.is_none() {
        anyhow::bail!("no settings specified; use --name, --paused");
    }
    Ok(())
}

fn device_display_name(device: &Device) -> String {
    device
        .display_name
        .as_deref()
        .or(device.nickname.as_deref())
        .or(device.hostname.as_deref())
        .or(device.mac.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

fn extract_url_id(url: &str) -> &str {
    url.rsplit('/').next().unwrap_or(url)
}

fn find_device_by_name<'a>(devices: &'a [Device], name: &str) -> Result<&'a Device> {
    let matches: Vec<_> = devices
        .iter()
        .filter(|d| {
            d.display_name.as_deref() == Some(name)
                || d.nickname.as_deref() == Some(name)
                || d.hostname.as_deref() == Some(name)
        })
        .collect();

    match matches.len() {
        0 => Err(anyhow::anyhow!("no device found with name {:?}", name)),
        1 => Ok(matches[0]),
        n => Err(anyhow::anyhow!(
            "{} devices found with name {:?}; use `id:XXXXX` to specify",
            n,
            name
        )),
    }
}

async fn resolve_device(
    client: &EeroClient,
    network_id: u64,
    device_arg: &str,
) -> Result<Device> {
    let devices = client.get_devices(network_id).await?;
    if let Some(id_str) = device_arg.strip_prefix("id:") {
        devices
            .into_iter()
            .find(|d| extract_url_id(&d.url) == id_str)
            .ok_or_else(|| anyhow::anyhow!("no device found with ID {:?}", id_str))
    } else {
        find_device_by_name(&devices, device_arg).cloned()
    }
}

fn find_eero_by_name<'a>(eeros: &'a [EeroNode], name: &str) -> Result<&'a EeroNode> {
    let matches: Vec<_> = eeros
        .iter()
        .filter(|e| e.location.as_deref() == Some(name))
        .collect();

    match matches.len() {
        0 => Err(anyhow::anyhow!("no eero found with location {:?}", name)),
        1 => Ok(matches[0]),
        n => Err(anyhow::anyhow!(
            "{} eeros found with location {:?}; use `id:XXXXX` to specify",
            n,
            name
        )),
    }
}

async fn resolve_eero(
    client: &EeroClient,
    network_id: u64,
    eero_arg: &str,
) -> Result<EeroNode> {
    let eeros = client.get_eeros(network_id).await?;
    if let Some(id_str) = eero_arg.strip_prefix("id:") {
        eeros
            .into_iter()
            .find(|e| extract_url_id(&e.url) == id_str)
            .ok_or_else(|| anyhow::anyhow!("no eero found with ID {:?}", id_str))
    } else {
        find_eero_by_name(&eeros, eero_arg).cloned()
    }
}

fn find_profile_by_name<'a>(profiles: &'a [Profile], name: &str) -> Result<&'a Profile> {
    let matches: Vec<_> = profiles
        .iter()
        .filter(|p| p.name.as_deref() == Some(name))
        .collect();

    match matches.len() {
        0 => Err(anyhow::anyhow!("no profile found with name {:?}", name)),
        1 => Ok(matches[0]),
        n => Err(anyhow::anyhow!(
            "{} profiles found with name {:?}; use `id:XXXXX` to specify",
            n,
            name
        )),
    }
}

async fn resolve_profile(
    client: &EeroClient,
    network_id: u64,
    profile_arg: &str,
) -> Result<Profile> {
    let profiles = client.get_profiles(network_id).await?;
    if let Some(id_str) = profile_arg.strip_prefix("id:") {
        profiles
            .into_iter()
            .find(|p| extract_url_id(&p.url) == id_str)
            .ok_or_else(|| anyhow::anyhow!("no profile found with ID {:?}", id_str))
    } else {
        find_profile_by_name(&profiles, profile_arg).cloned()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let env_filter = if let Some(ref level) = cli.log_level {
        tracing_subscriber::EnvFilter::new(level)
    } else {
        tracing_subscriber::EnvFilter::from_default_env()
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
    let out = Output::new(cli.json, cli.no_color, cli.jq.clone());
    let yes = cli.yes;

    let client = if let Some(ref token) = cli.user_token {
        let store = InMemoryStore::new();
        let client = ClientBuilder::new()
            .credential_store(Box::new(store))
            .build()?;
        client.credentials().set_session_token(token).await?;
        client
    } else {
        EeroClient::new()?
    };

    match cli.command {
        Command::Auth(cmd) => match cmd {
            AuthCmd::Login { email_or_phone } => {
                client.login(&email_or_phone).await?;
                eprintln!("Verification code sent. Run `eeroctl auth verify <code>` next.");
            }
            AuthCmd::Verify { code } => {
                client.verify(&code).await?;
                out.print_ok()?;
            }
            AuthCmd::Logout => {
                confirm(
                    "end this session for every machine that shares it",
                    yes,
                )?;
                client.logout().await?;
                out.print_ok()?;
            }
        },
        Command::Account(cmd) => match cmd {
            AccountCmd::Get => {
                let account = client.get_account().await?;
                out.print(&account)?;
            }
        },
        Command::Network(cmd) => match cmd {
            NetworkCmd::Get { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let network = client.get_network(network_id).await?;
                out.print(&network)?;
            }
            NetworkCmd::GetByUrl { url } => {
                let network = client.get_network_by_url(&url).await?;
                out.print(&network)?;
            }
            NetworkCmd::Set {
                network,
                name,
                password,
                sqm,
                band_steering,
                thread,
                upnp,
                ipv6,
                wpa3,
            } => {
                let update = NetworkUpdate {
                    name,
                    password,
                    sqm,
                    band_steering,
                    thread,
                    upnp,
                    ipv6,
                    wpa3,
                    ..Default::default()
                };
                require_network_set_flags(&update)?;
                if update.password.is_some() {
                    confirm(
                        "change the Wi-Fi password and disconnect every wireless device",
                        yes,
                    )?;
                }
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let result = client.update_network(network_id, &update).await?;
                out.print(&result)?;
            }
            NetworkCmd::Reboot { network } => {
                confirm(
                    &format!(
                        "reboot every eero on {}",
                        network.as_deref().unwrap_or("the default network")
                    ),
                    yes,
                )?;
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let resp = client.reboot_network(network_id).await?;
                out.print(&resp)?;
            }
            NetworkCmd::SpeedTest(cmd) => match cmd {
                SpeedTestCmd::Run { network } => {
                    let network_id =
                        resolve_network_id_or_default(&client, network.as_deref()).await?;
                    client.run_speed_test(network_id).await?;
                    eprintln!("Speed test started. Please run `speed-test get` in a moment.");
                }
                SpeedTestCmd::Get { network } => {
                    let network_id =
                        resolve_network_id_or_default(&client, network.as_deref()).await?;
                    let result = client.get_speed_test(network_id).await?;
                    out.print(&result)?;
                }
            },
            NetworkCmd::DataUsage {
                network,
                start,
                end,
                cadence,
                timezone,
                by,
            } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let query = DataUsageQuery {
                    start,
                    end,
                    cadence,
                    timezone,
                };
                match by {
                    None => out.print(&client.get_data_usage(network_id, &query).await?)?,
                    Some(UsageBreakdown::Devices) => {
                        out.print(&client.get_device_data_usage(network_id, &query).await?)?
                    }
                    Some(UsageBreakdown::Eeros) => {
                        out.print(&client.get_eero_data_usage(network_id, &query).await?)?
                    }
                }
            }
            NetworkCmd::Guest(cmd) => match cmd {
                GuestCmd::Get { network } => {
                    let network_id =
                        resolve_network_id_or_default(&client, network.as_deref()).await?;
                    let guest = client.get_guest_network(network_id).await?;
                    out.print(&guest)?;
                }
                GuestCmd::Set {
                    network,
                    name,
                    password,
                    enabled,
                } => {
                    let update = GuestNetworkUpdate {
                        name,
                        password,
                        enabled,
                    };
                    require_guest_set_flags(&update)?;
                    let network_id =
                        resolve_network_id_or_default(&client, network.as_deref()).await?;
                    let guest = client.update_guest_network(network_id, &update).await?;
                    out.print(&guest)?;
                }
            },
        },
        Command::Eero(cmd) => match cmd {
            EeroCmd::List { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let eeros = client.get_eeros(network_id).await?;
                out.print(&eeros)?;
            }
            EeroCmd::Get { eero, network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let eero_node = resolve_eero(&client, network_id, &eero).await?;
                let full = client.get_eero(&eero_node.url).await?;
                out.print(&full)?;
            }
            EeroCmd::Set {
                eero,
                network,
                location,
                led_on,
                nightlight_enabled,
                nightlight_brightness,
                nightlight_on,
                nightlight_off,
            } => {
                let nightlight = if nightlight_enabled.is_some()
                    || nightlight_brightness.is_some()
                    || nightlight_on.is_some()
                    || nightlight_off.is_some()
                {
                    let schedule = if nightlight_on.is_some() || nightlight_off.is_some() {
                        Some(NightlightScheduleUpdate {
                            on: nightlight_on,
                            off: nightlight_off,
                        })
                    } else {
                        None
                    };
                    Some(NightlightUpdate {
                        enabled: nightlight_enabled,
                        brightness: nightlight_brightness,
                        schedule,
                    })
                } else {
                    None
                };
                let update = EeroUpdate {
                    location,
                    led_on,
                    nightlight,
                };
                require_eero_set_flags(&update)?;
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let eero_node = resolve_eero(&client, network_id, &eero).await?;
                let result = client.update_eero(&eero_node.url, &update).await?;
                out.print(&result)?;
            }
            EeroCmd::Reboot { eero, network } => {
                confirm(&format!("reboot the eero {eero:?}"), yes)?;
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let eero_node = resolve_eero(&client, network_id, &eero).await?;
                let resp = client.reboot_eero(&eero_node.url).await?;
                out.print(&resp)?;
            }
        },
        Command::Device(cmd) => match cmd {
            DeviceCmd::List { network, machine } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let devices = client.get_devices(network_id).await?;
                if machine || out.jq_filter.is_some() {
                    let summary: Vec<serde_json::Value> = devices
                        .iter()
                        .map(|d| {
                            serde_json::json!({
                                "id": extract_url_id(&d.url),
                                "name": device_display_name(d),
                            })
                        })
                        .collect();
                    out.print(&summary)?;
                } else {
                    let mut stdout = io::stdout().lock();
                    writeln!(stdout, "{:<12} NAME", "ID")?;
                    for d in &devices {
                        writeln!(
                            stdout,
                            "{:<12} {}",
                            extract_url_id(&d.url),
                            device_display_name(d),
                        )?;
                    }
                }
            }
            DeviceCmd::Get { device, network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let dev = resolve_device(&client, network_id, &device).await?;
                let full = client.get_device(&dev.url).await?;
                out.print(&full)?;
            }
            DeviceCmd::Set {
                device,
                network,
                nickname,
                blocked,
                paused,
                profile,
            } => {
                let update = DeviceUpdate {
                    nickname,
                    blocked,
                    paused,
                    profile,
                };
                require_device_set_flags(&update)?;
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let dev = resolve_device(&client, network_id, &device).await?;
                let device = client.update_device(&dev.url, &update).await?;
                out.print(&device)?;
            }
        },
        Command::Profile(cmd) => match cmd {
            ProfileCmd::List { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let profiles = client.get_profiles(network_id).await?;
                out.print(&profiles)?;
            }
            ProfileCmd::Get { profile, network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let prof = resolve_profile(&client, network_id, &profile).await?;
                let full = client.get_profile(&prof.url).await?;
                out.print(&full)?;
            }
            ProfileCmd::Set {
                profile,
                network,
                name,
                paused,
            } => {
                let update = ProfileUpdate {
                    name,
                    paused,
                    ..Default::default()
                };
                require_profile_set_flags(&update)?;
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let prof = resolve_profile(&client, network_id, &profile).await?;
                let result = client.update_profile(&prof.url, &update).await?;
                out.print(&result)?;
            }
        },
        Command::Forward(cmd) => match cmd {
            ForwardCmd::List { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let forwards = client.get_port_forwards(network_id).await?;
                out.print(&forwards)?;
            }
            ForwardCmd::Create {
                network,
                ip_address,
                port,
                external_port,
                protocol,
            } => {
                let forward = NewPortForward {
                    ip_address,
                    port,
                    external_port,
                    protocol,
                };
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let result = client.create_port_forward(network_id, &forward).await?;
                out.print(&result)?;
            }
            ForwardCmd::Delete { forward_id } => {
                confirm(&format!("delete port forward {forward_id}"), yes)?;
                let url = format!("/2.2/forwards/{}", forward_id);
                let resp = client.delete_port_forward(&url).await?;
                out.print(&resp)?;
            }
        },
        Command::Reservation(cmd) => match cmd {
            ReservationCmd::List { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                out.print(&client.get_reservations(network_id).await?)?;
            }
            ReservationCmd::Create {
                network,
                ip,
                mac,
                description,
            } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let reservation = NewReservation {
                    ip,
                    mac,
                    description,
                };
                out.print(&client.create_reservation(network_id, &reservation).await?)?;
            }
            ReservationCmd::Delete {
                reservation,
                network,
            } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let reservations = client.get_reservations(network_id).await?;
                let found = find_reservation(&reservations, &reservation)?;
                confirm(
                    &format!(
                        "delete the reservation of {} for {}",
                        found.ip.as_deref().unwrap_or("?"),
                        found.mac.as_deref().unwrap_or("?")
                    ),
                    yes,
                )?;
                out.print(&client.delete_reservation(&found.url).await?)?;
            }
        },
        Command::Activity(cmd) => match cmd {
            ActivityCmd::Get { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let activity = client.get_activity(network_id).await?;
                out.print(&activity)?;
            }
            ActivityCmd::GetClient { device, network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let dev = resolve_device(&client, network_id, &device).await?;
                let activity = client.get_client_activity(&dev.url).await?;
                out.print(&activity)?;
            }
        },
        Command::Diagnostics(cmd) => match cmd {
            DiagnosticsCmd::Get { network } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let report = client.get_diagnostics(network_id).await?;
                out.print(&report)?;
            }
            DiagnosticsCmd::Run {
                network,
                symptom,
                device,
            } => {
                let network_id =
                    resolve_network_id_or_default(&client, network.as_deref()).await?;
                let device = match device {
                    Some(arg) => {
                        let found = resolve_device(&client, network_id, &arg).await?;
                        Some(found.mac.unwrap_or_else(|| extract_url_id(&found.url).to_string()))
                    }
                    None => None,
                };
                let request = DiagnosticsRequest { symptom, device };
                let report = client.run_diagnostics(network_id, &request).await?;
                out.print(&report)?;
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_confirm_with() {
        let ask = |yes: bool, interactive: bool, answer: &str| {
            let mut prompt = Vec::new();
            let result = confirm_with(
                "reboot the mesh",
                yes,
                interactive,
                &mut answer.as_bytes(),
                &mut prompt,
            );
            (result.map_err(|e| e.to_string()), String::from_utf8(prompt).unwrap())
        };

        assert_eq!(ask(true, false, ""), (Ok(()), String::new()));
        let (result, prompt) = ask(false, false, "y\n");
        assert!(result.unwrap_err().contains("--yes"));
        assert!(prompt.is_empty());
        let (result, prompt) = ask(false, true, "y\n");
        assert!(result.is_ok());
        assert_eq!(prompt, "This will reboot the mesh. Continue? [y/N] ");
        assert!(ask(false, true, "YES\n").0.is_ok());
        assert_eq!(ask(false, true, "\n").0, Err("aborted".to_string()));
        assert_eq!(ask(false, true, "nope\n").0, Err("aborted".to_string()));
    }

    fn reservations() -> Vec<Reservation> {
        serde_json::from_value(json!([
            {"url": "/2.2/networks/1/reservations/5001", "ip": "192.168.4.50", "mac": "11:22:33:44:55:66"},
            {"url": "/2.2/networks/1/reservations/5002", "ip": "192.168.4.51", "mac": "AA:BB:CC:DD:EE:11"}
        ]))
        .unwrap()
    }

    #[test]
    fn test_find_reservation_by_id_ip_and_mac() {
        let all = reservations();
        let id = |arg: &str| extract_url_id(&find_reservation(&all, arg).unwrap().url).to_string();
        assert_eq!(id("id:5002"), "5002");
        assert_eq!(id("192.168.4.50"), "5001");
        assert_eq!(id("aa-bb-cc-dd-ee-11"), "5002");
        assert_eq!(id("11:22:33:44:55:66"), "5001");
        assert!(find_reservation(&all, "192.168.4.99").is_err());
        assert!(find_reservation(&all, "id:9999").is_err());
    }

    #[test]
    fn test_is_default() {
        assert!(is_default(&json!(null)));
        assert!(is_default(&json!(false)));
        assert!(is_default(&json!(0)));
        assert!(is_default(&json!(0.0)));
        assert!(is_default(&json!("")));
        assert!(is_default(&json!([])));
        assert!(is_default(&json!({})));

        assert!(!is_default(&json!(true)));
        assert!(!is_default(&json!(1)));
        assert!(!is_default(&json!("hello")));
        assert!(!is_default(&json!([1])));
        assert!(!is_default(&json!({"a": 1})));
    }

    #[test]
    fn test_strip_defaults_flat() {
        let input = json!({
            "name": "test",
            "description": "",
            "count": 0,
            "enabled": false,
            "tags": [],
            "meta": {},
            "value": null,
            "active": true,
            "score": 42
        });
        let result = strip_defaults(input);
        assert_eq!(
            result,
            json!({
                "name": "test",
                "active": true,
                "score": 42
            })
        );
    }

    #[test]
    fn test_strip_defaults_nested() {
        let input = json!({
            "name": "router",
            "settings": {
                "dns": "",
                "upnp": false,
                "ports": {
                    "list": [],
                    "enabled": null
                }
            },
            "location": "office"
        });
        let result = strip_defaults(input);
        // "settings" and its children all resolve to defaults, so the whole subtree is removed
        assert_eq!(
            result,
            json!({
                "name": "router",
                "location": "office"
            })
        );
    }

    #[test]
    fn test_strip_defaults_array_elements() {
        let input = json!([
            {"name": "a", "value": null},
            {"name": "", "active": false},
            {"name": "b", "score": 5}
        ]);
        let result = strip_defaults(input);
        assert_eq!(
            result,
            json!([
                {"name": "a"},
                {},
                {"name": "b", "score": 5}
            ])
        );
    }

    #[test]
    fn test_parse_network_arg_literal() {
        assert_eq!(
            parse_network_arg("id:12345").unwrap(),
            NetworkArg::Literal(12345)
        );
    }

    #[test]
    fn test_parse_network_arg_literal_invalid() {
        assert!(parse_network_arg("id:abc").is_err());
    }

    #[test]
    fn test_parse_network_arg_ssid() {
        assert_eq!(
            parse_network_arg("MyWiFi").unwrap(),
            NetworkArg::Ssid("MyWiFi".to_string())
        );
    }

    #[test]
    fn test_extract_network_id_valid() {
        assert_eq!(
            extract_network_id("/2.2/networks/12345").unwrap(),
            12345
        );
    }

    #[test]
    fn test_extract_network_id_invalid() {
        assert!(extract_network_id("/2.2/networks/").is_err());
        assert!(extract_network_id("").is_err());
    }

    #[test]
    fn test_find_network_by_ssid_found() {
        let networks = vec![
            NetworkRef {
                url: "/2.2/networks/111".to_string(),
                name: Some("Home".to_string()),
            },
            NetworkRef {
                url: "/2.2/networks/222".to_string(),
                name: Some("Office".to_string()),
            },
        ];
        assert_eq!(find_network_by_ssid(&networks, "Office").unwrap(), 222);
    }

    #[test]
    fn test_find_network_by_ssid_not_found() {
        let networks = vec![NetworkRef {
            url: "/2.2/networks/111".to_string(),
            name: Some("Home".to_string()),
        }];
        assert!(find_network_by_ssid(&networks, "Office").is_err());
    }

    #[test]
    fn test_find_network_by_ssid_multiple() {
        let networks = vec![
            NetworkRef {
                url: "/2.2/networks/111".to_string(),
                name: Some("Dupe".to_string()),
            },
            NetworkRef {
                url: "/2.2/networks/222".to_string(),
                name: Some("Dupe".to_string()),
            },
        ];
        assert!(find_network_by_ssid(&networks, "Dupe").is_err());
    }

    #[test]
    fn test_strip_defaults_preserves_non_defaults() {
        let input = json!({
            "a": 1,
            "b": "hello",
            "c": true,
            "d": [1, 2],
            "e": {"nested": "value"}
        });
        let result = strip_defaults(input.clone());
        assert_eq!(result, input);
    }

    fn make_device(url: &str, display_name: Option<&str>, nickname: Option<&str>, hostname: Option<&str>, mac: Option<&str>) -> Device {
        Device {
            url: url.to_string(),
            display_name: display_name.map(|s| s.to_string()),
            nickname: nickname.map(|s| s.to_string()),
            hostname: hostname.map(|s| s.to_string()),
            mac: mac.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn make_eero(url: &str, location: Option<&str>) -> EeroNode {
        EeroNode {
            url: url.to_string(),
            location: location.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn make_profile(url: &str, name: Option<&str>) -> Profile {
        Profile {
            url: url.to_string(),
            name: name.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_url_id() {
        assert_eq!(extract_url_id("/2.2/devices/99999"), "99999");
        assert_eq!(extract_url_id("/2.2/networks/12345"), "12345");
        assert_eq!(extract_url_id("simple"), "simple");
    }

    #[test]
    fn test_device_display_name_priority() {
        let d = make_device("/d/1", Some("Display"), Some("Nick"), Some("host"), Some("AA:BB"));
        assert_eq!(device_display_name(&d), "Display");

        let d = make_device("/d/1", None, Some("Nick"), Some("host"), Some("AA:BB"));
        assert_eq!(device_display_name(&d), "Nick");

        let d = make_device("/d/1", None, None, Some("host"), Some("AA:BB"));
        assert_eq!(device_display_name(&d), "host");

        let d = make_device("/d/1", None, None, None, Some("AA:BB"));
        assert_eq!(device_display_name(&d), "AA:BB");

        let d = make_device("/d/1", None, None, None, None);
        assert_eq!(device_display_name(&d), "unknown");
    }

    #[test]
    fn test_find_device_by_name_display_name() {
        let devices = vec![
            make_device("/d/1", Some("Phone"), None, None, None),
            make_device("/d/2", Some("Laptop"), None, None, None),
        ];
        let found = find_device_by_name(&devices, "Laptop").unwrap();
        assert_eq!(found.url, "/d/2");
    }

    #[test]
    fn test_find_device_by_name_hostname() {
        let devices = vec![
            make_device("/d/1", None, None, Some("myhost"), None),
        ];
        let found = find_device_by_name(&devices, "myhost").unwrap();
        assert_eq!(found.url, "/d/1");
    }

    #[test]
    fn test_find_device_by_name_not_found() {
        let devices = vec![
            make_device("/d/1", Some("Phone"), None, None, None),
        ];
        assert!(find_device_by_name(&devices, "Tablet").is_err());
    }

    #[test]
    fn test_find_device_by_name_multiple() {
        let devices = vec![
            make_device("/d/1", Some("Dupe"), None, None, None),
            make_device("/d/2", Some("Dupe"), None, None, None),
        ];
        assert!(find_device_by_name(&devices, "Dupe").is_err());
    }

    #[test]
    fn test_find_eero_by_name_found() {
        let eeros = vec![
            make_eero("/e/1", Some("Living Room")),
            make_eero("/e/2", Some("Bedroom")),
        ];
        let found = find_eero_by_name(&eeros, "Bedroom").unwrap();
        assert_eq!(found.url, "/e/2");
    }

    #[test]
    fn test_find_eero_by_name_not_found() {
        let eeros = vec![
            make_eero("/e/1", Some("Living Room")),
        ];
        assert!(find_eero_by_name(&eeros, "Kitchen").is_err());
    }

    #[test]
    fn test_find_eero_by_name_multiple() {
        let eeros = vec![
            make_eero("/e/1", Some("Dupe")),
            make_eero("/e/2", Some("Dupe")),
        ];
        assert!(find_eero_by_name(&eeros, "Dupe").is_err());
    }

    #[test]
    fn test_find_profile_by_name_found() {
        let profiles = vec![
            make_profile("/p/1", Some("Default")),
            make_profile("/p/2", Some("Kids")),
        ];
        let found = find_profile_by_name(&profiles, "Kids").unwrap();
        assert_eq!(found.url, "/p/2");
    }

    #[test]
    fn test_find_profile_by_name_not_found() {
        let profiles = vec![
            make_profile("/p/1", Some("Default")),
        ];
        assert!(find_profile_by_name(&profiles, "Guests").is_err());
    }

    #[test]
    fn test_find_profile_by_name_multiple() {
        let profiles = vec![
            make_profile("/p/1", Some("Dupe")),
            make_profile("/p/2", Some("Dupe")),
        ];
        assert!(find_profile_by_name(&profiles, "Dupe").is_err());
    }

    #[test]
    fn test_require_network_set_flags_empty() {
        let update = NetworkUpdate::default();
        assert!(require_network_set_flags(&update).is_err());
    }

    #[test]
    fn test_require_network_set_flags_with_name() {
        let update = NetworkUpdate {
            name: Some("NewName".to_string()),
            ..Default::default()
        };
        assert!(require_network_set_flags(&update).is_ok());
    }

    #[test]
    fn test_require_guest_set_flags_empty() {
        let update = GuestNetworkUpdate {
            name: None,
            password: None,
            enabled: None,
        };
        assert!(require_guest_set_flags(&update).is_err());
    }

    #[test]
    fn test_require_guest_set_flags_with_enabled() {
        let update = GuestNetworkUpdate {
            name: None,
            password: None,
            enabled: Some(false),
        };
        assert!(require_guest_set_flags(&update).is_ok());
    }

    #[test]
    fn test_require_eero_set_flags_empty() {
        let update = EeroUpdate::default();
        assert!(require_eero_set_flags(&update).is_err());
    }

    #[test]
    fn test_require_eero_set_flags_with_location() {
        let update = EeroUpdate {
            location: Some("Office".to_string()),
            ..Default::default()
        };
        assert!(require_eero_set_flags(&update).is_ok());
    }

    #[test]
    fn test_require_eero_set_flags_with_nightlight() {
        let update = EeroUpdate {
            nightlight: Some(NightlightUpdate {
                enabled: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(require_eero_set_flags(&update).is_ok());
    }

    #[test]
    fn test_require_device_set_flags_empty() {
        let update = DeviceUpdate::default();
        assert!(require_device_set_flags(&update).is_err());
    }

    #[test]
    fn test_require_device_set_flags_with_nickname() {
        let update = DeviceUpdate {
            nickname: Some("MyDevice".to_string()),
            ..Default::default()
        };
        assert!(require_device_set_flags(&update).is_ok());
    }

    #[test]
    fn test_require_profile_set_flags_empty() {
        let update = ProfileUpdate::default();
        assert!(require_profile_set_flags(&update).is_err());
    }

    #[test]
    fn test_require_profile_set_flags_with_paused() {
        let update = ProfileUpdate {
            paused: Some(true),
            ..Default::default()
        };
        assert!(require_profile_set_flags(&update).is_ok());
    }

    #[test]
    fn test_jq_filter_identity() {
        let out = Output::new(false, true, Some(".".to_string()));
        // Should not panic — just run the filter
        let val = json!({"name": "test"});
        // We can't easily capture stdout in a unit test, but we can verify it doesn't error
        out.print(&val).unwrap();
    }

    #[test]
    fn test_jq_filter_field() {
        let out = Output::new(false, true, Some(".name".to_string()));
        let val = json!({"name": "hello", "other": 42});
        out.print(&val).unwrap();
    }

    #[test]
    fn test_jq_filter_invalid() {
        let out = Output::new(false, true, Some("invalid[[[".to_string()));
        let val = json!({"name": "test"});
        assert!(out.print(&val).is_err());
    }
}
