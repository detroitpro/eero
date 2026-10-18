use serde_json::{json, Value};

pub fn account() -> Value {
    json!({
        "url": "/2.2/account",
        "name": "Test User",
        "phone": { "value": "+15555550100", "verified": true },
        "email": { "value": "test@example.com", "verified": true },
        "networks": {
            "count": 1,
            "data": [
                {
                    "url": "/2.2/networks/12345",
                    "name": "TestNetwork"
                }
            ]
        },
        "premium_status": "active",
        "can_transfer": true,
        "is_premium": true,
        "role": "owner"
    })
}

pub fn network() -> Value {
    json!({
        "url": "/2.2/networks/12345",
        "name": "TestNetwork",
        "password": "testpass123",
        "health": {
            "internet": { "status": "connected" },
            "eero_network": { "status": "connected" }
        },
        "speed": {
            "down": { "value": 500.0, "units": "Mbps" },
            "up": { "value": 50.0, "units": "Mbps" }
        },
        "clients": { "count": 10, "url": "/2.2/networks/12345/devices" },
        "eeros": { "count": 3, "url": "/2.2/networks/12345/eeros" },
        "profiles": { "count": 2, "url": "/2.2/networks/12345/profiles" },
        "dns": {
            "custom": false,
            "parent": "cloudflare",
            "provider": "cloudflare",
            "caching": true
        },
        "guest_network": {
            "name": "TestNetwork Guest",
            "password": "guestpass",
            "enabled": true,
            "url": "/2.2/networks/12345/guestnetwork"
        },
        "sqm": true,
        "band_steering": true,
        "thread": false,
        "upnp": true,
        "ipv6": true,
        "wpa3": true,
        "target_firmware": "7.0.0",
        "updates": { "enabled": true, "schedule": "auto" },
        "timezone": { "value": "America/Los_Angeles" }
    })
}

pub fn eero_node() -> Value {
    json!({
        "url": "/2.2/eeros/1001",
        "serial": "F01234567890",
        "model": "eero Pro 6E",
        "location": "Living Room",
        "joined": "2024-01-15T10:00:00Z",
        "os_version": "7.0.0-123",
        "status": "green",
        "status_light": { "enabled": true, "brightness": 100 },
        "connected_clients_count": 5,
        "nightlight": {
            "enabled": false,
            "schedule": { "on": "21:00", "off": "07:00" },
            "brightness": 50
        },
        "gateway": true,
        "ip_address": "192.168.4.1",
        "mesh_quality_bars": 5,
        "wired": true,
        "ethernet_addresses": [{ "mac": "AA:BB:CC:DD:EE:FF" }],
        "update_available": false
    })
}

pub fn eero_node_2() -> Value {
    json!({
        "url": "/2.2/eeros/1002",
        "serial": "F09876543210",
        "model": "eero 6+",
        "location": "Bedroom",
        "joined": "2024-01-15T10:05:00Z",
        "os_version": "7.0.0-123",
        "status": "green",
        "connected_clients_count": 3,
        "gateway": false,
        "ip_address": "192.168.4.2",
        "mesh_quality_bars": 4,
        "wired": false,
        "update_available": false
    })
}

pub fn device() -> Value {
    json!({
        "url": "/2.2/devices/2001",
        "mac": "11:22:33:44:55:66",
        "manufacturer": "Apple",
        "ip": "192.168.4.100",
        "ips": ["192.168.4.100"],
        "nickname": "MacBook Pro",
        "hostname": "macbook-pro",
        "display_name": "MacBook Pro",
        "connected": true,
        "wireless": true,
        "connection_type": "wireless",
        "source": {
            "location": "Living Room",
            "url": "/2.2/eeros/1001"
        },
        "profile": {
            "url": "/2.2/profiles/3001",
            "name": "Default"
        },
        "paused": false,
        "blocked": false,
        "is_guest": false,
        "is_private": false,
        "frequency": "5 GHz",
        "frequency_band": "5",
        "auth": "wpa3",
        "usage": { "down_mbps": 12.5, "up_mbps": 3.25 }
    })
}

pub fn device_2() -> Value {
    json!({
        "url": "/2.2/devices/2002",
        "mac": "AA:BB:CC:DD:EE:11",
        "manufacturer": "Samsung",
        "ip": "192.168.4.101",
        "nickname": null,
        "hostname": "galaxy-phone",
        "display_name": "galaxy-phone",
        "connected": true,
        "wireless": true,
        "connection_type": "wireless",
        "paused": false,
        "blocked": false,
        "is_guest": false
    })
}

pub fn profile() -> Value {
    json!({
        "url": "/2.2/profiles/3001",
        "name": "Default",
        "paused": false,
        "blocked_applications": [],
        "dns_policies": {
            "block": [],
            "safe_search_enabled": false,
            "youtube_restricted": false
        },
        "devices": [
            {
                "url": "/2.2/devices/2001",
                "mac": "11:22:33:44:55:66",
                "display_name": "MacBook Pro"
            }
        ],
        "schedule": { "enabled": false, "time_ranges": [] }
    })
}

pub fn profile_2() -> Value {
    json!({
        "url": "/2.2/profiles/3002",
        "name": "Kids",
        "paused": false,
        "blocked_applications": ["tiktok"],
        "dns_policies": {
            "block": ["adult"],
            "safe_search_enabled": true,
            "youtube_restricted": true
        },
        "devices": [],
        "schedule": { "enabled": true, "time_ranges": [] }
    })
}

pub fn port_forward() -> Value {
    json!({
        "url": "/2.2/forwards/4001",
        "enabled": true,
        "ip_address": "192.168.4.100",
        "port": "8080",
        "external_port": "80",
        "protocol": "tcp",
        "hostname": "macbook-pro",
        "display_name": "Web Server"
    })
}

pub fn reservations() -> Value {
    json!([
        {
            "url": "/2.2/networks/12345/reservations/5001",
            "ip": "192.168.4.50",
            "mac": "11:22:33:44:55:66",
            "description": "NAS"
        },
        {
            "url": "/2.2/networks/12345/reservations/5002",
            "ip": "192.168.4.51",
            "mac": "AA:BB:CC:DD:EE:11",
            "description": "Printer"
        }
    ])
}

pub fn speed_test() -> Value {
    json!([
        {
            "date": "2024-06-15T12:00:00Z",
            "up_mbps": 48.5,
            "down_mbps": 495.2
        }
    ])
}

pub fn guest_network() -> Value {
    json!({
        "name": "TestNetwork Guest",
        "password": "guestpass",
        "enabled": true,
        "url": "/2.2/networks/12345/guestnetwork"
    })
}

pub fn activity() -> Value {
    json!({
        "insights": [
            {
                "type": "top_domain",
                "category": "streaming",
                "count": 1500,
                "domain": "netflix.com"
            },
            {
                "type": "top_domain",
                "category": "social",
                "count": 800,
                "domain": "twitter.com"
            }
        ],
        "counts": {
            "blocked": 42,
            "allowed": 15000,
            "inspected": 15042
        }
    })
}

pub fn client_activity() -> Value {
    json!({
        "url": "/2.2/devices/2001",
        "display_name": "MacBook Pro",
        "insights": [
            {
                "type": "top_domain",
                "category": "productivity",
                "count": 500,
                "domain": "github.com"
            }
        ],
        "counts": {
            "blocked": 5,
            "allowed": 3000,
            "inspected": 3005
        }
    })
}

pub fn data_usage() -> Value {
    json!({
        "start": "2026-08-24T04:00:00Z",
        "end": "2026-08-25T03:59:59Z",
        "limit": "2026-04-30T00:00:00Z",
        "series": [
            {
                "type": "upload",
                "sum": 300,
                "values": [{ "time": "2026-08-24T04:00:00Z", "value": 300 }]
            },
            {
                "type": "download",
                "sum": 1200,
                "values": [{ "time": "2026-08-24T04:00:00Z", "value": 1200 }]
            }
        ]
    })
}

pub fn device_data_usage() -> Value {
    json!({
        "start": "2026-08-24T04:00:00Z",
        "end": "2026-08-25T03:59:59Z",
        "values": [
            {
                "url": "/2.2/devices/2001",
                "mac": "11:22:33:44:55:66",
                "display_name": "MacBook Pro",
                "download": 1200,
                "upload": 300
            }
        ]
    })
}

pub fn eero_data_usage() -> Value {
    json!({
        "start": "2026-08-24T04:00:00Z",
        "end": "2026-08-25T03:59:59Z",
        "values": [
            {
                "url": "/2.2/eeros/1001",
                "location": "Living Room",
                "download": 1200,
                "upload": 300
            }
        ]
    })
}

pub fn diagnostics() -> Value {
    json!({
        "url": "/2.2/networks/12345/diagnostics/6001",
        "status": "completed",
        "created": "2024-06-15T12:00:00Z",
        "completed": "2024-06-15T12:01:00Z",
        "results": [
            {
                "type": "internet",
                "status": "pass",
                "details": { "latency_ms": 12 }
            },
            {
                "type": "dns",
                "status": "pass",
                "details": { "resolver": "1.1.1.1" }
            }
        ]
    })
}

pub fn login_response() -> Value {
    json!({
        "user_token": "ut_test_token_12345"
    })
}
