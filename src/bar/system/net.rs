// ─── < Imports > ────────────────────────────────────────────────────

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::state::WifiInfo;

// ─── < Structs > ────────────────────────────────────────────────────

/// Lo que `iw dev <if> link` cuenta de la conexión actual.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LinkStatus {
    pub ssid: Option<String>,
    pub signal_dbm: Option<i32>,
    pub freq_mhz: Option<u32>,
}

// ─── < Public Functions: Parsers > ────────────────────────────────────────────────────

/// Interfaz y gateway de la ruta por defecto en `/proc/net/route`
/// (destino 00000000; el gateway viene en hexa little-endian).
pub fn parse_default_route(route: &str) -> Option<(String, String)> {
    for line in route.lines().skip(1) {
        let mut fields = line.split_whitespace();

        let interface = fields.next()?;
        let destination = fields.next()?;
        let gateway_hex = fields.next()?;

        if destination != "00000000" {
            continue;
        }

        let raw = u32::from_str_radix(gateway_hex, 16).ok()?;
        let octets = raw.to_le_bytes();
        let gateway = format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);

        return Some((interface.to_owned(), gateway));
    }

    None
}

/// Bytes recibidos/enviados de una interfaz según `/proc/net/dev`.
pub fn parse_interface_bytes(net_dev: &str, interface: &str) -> Option<(u64, u64)> {
    for line in net_dev.lines() {
        let Some((name, rest)) = line.split_once(':') else {
            continue;
        };

        if name.trim() != interface {
            continue;
        }

        let fields: Vec<&str> = rest.split_whitespace().collect();
        let rx = fields.first()?.parse().ok()?;
        let tx = fields.get(8)?.parse().ok()?;

        return Some((rx, tx));
    }

    None
}

/// Salida de `iw dev <if> link`: SSID, señal en dBm y frecuencia.
pub fn parse_iw_link(output: &str) -> LinkStatus {
    let mut status = LinkStatus::default();

    for line in output.lines() {
        let line = line.trim();

        if let Some(value) = line.strip_prefix("SSID:") {
            status.ssid = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("signal:") {
            status.signal_dbm = value.split_whitespace().next().and_then(|v| v.parse().ok());
        } else if let Some(value) = line.strip_prefix("freq:") {
            // Puede venir "5220" o "5220.0".
            status.freq_mhz = value.trim().split('.').next().and_then(|v| v.parse().ok());
        }
    }

    status
}

/// Banda legible para una frecuencia en MHz.
pub fn band_label(freq_mhz: u32) -> &'static str {
    match freq_mhz {
        0..=3000 => "2.4 GHz",
        3001..=5925 => "5 GHz",
        _ => "6 GHz",
    }
}

/// Canal wifi a partir de la frecuencia en MHz.
pub fn channel_for_freq(freq_mhz: u32) -> Option<u32> {
    match freq_mhz {
        2412..=2472 => Some((freq_mhz - 2407) / 5),
        2484 => Some(14),
        5180..=5925 => Some((freq_mhz - 5000) / 5),
        5955..=7115 => Some((freq_mhz - 5950) / 5),
        _ => None,
    }
}

/// Seguridad de la red activa según `nmcli -t -f active,security dev wifi`
/// (toma el mecanismo más fuerte listado, p. ej. "WPA2 WPA3" → "WPA3").
pub fn parse_active_security(output: &str) -> Option<String> {
    for line in output.lines() {
        let Some((active, security)) = line.split_once(':') else {
            continue;
        };

        if active.trim() != "yes" && active.trim() != "sí" {
            continue;
        }

        return security.split_whitespace().last().map(str::to_owned);
    }

    None
}

/// Primera dirección IPv4 con prefijo en la salida de
/// `ip -o -4 addr show dev <if>` (campo después de "inet").
pub fn parse_ipv4_with_prefix(output: &str) -> Option<String> {
    let mut fields = output.split_whitespace();

    while let Some(field) = fields.next() {
        if field == "inet" {
            return fields.next().map(str::to_owned);
        }
    }

    None
}

/// Nameservers de un resolv.conf.
pub fn parse_nameservers(resolv: &str) -> Vec<String> {
    resolv
        .lines()
        .filter_map(|line| line.trim().strip_prefix("nameserver"))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

/// RTT en ms de una salida de `ping -c 1` ("time=34.2 ms").
pub fn parse_ping_ms(output: &str) -> Option<f32> {
    let index = output.find("time=")?;
    let rest = &output[index + "time=".len()..];

    rest.split_whitespace().next().and_then(|value| value.parse().ok())
}

// ─── < Public Functions: Lecturas > ────────────────────────────────────────────────────

pub fn read_default_route() -> Result<(String, String)> {
    let route = fs::read_to_string("/proc/net/route").context("leyendo /proc/net/route")?;

    parse_default_route(&route).context("sin ruta por defecto")
}

pub fn read_interface_bytes(interface: &str) -> Result<(u64, u64)> {
    let net_dev = fs::read_to_string("/proc/net/dev").context("leyendo /proc/net/dev")?;

    parse_interface_bytes(&net_dev, interface).context("interfaz sin contadores en /proc/net/dev")
}

pub fn is_wireless(interface: &str) -> bool {
    Path::new("/sys/class/net").join(interface).join("wireless").exists()
}

/// Info wifi de la interfaz activa (iw + nmcli, ambos tolerados ausentes).
pub fn read_wifi_info(interface: &str) -> Option<WifiInfo> {
    let output = Command::new("iw").args(["dev", interface, "link"]).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let link = parse_iw_link(&stdout);

    let ssid = link.ssid?;

    let security = Command::new("nmcli")
        .args(["-t", "-f", "active,security", "dev", "wifi"])
        .output()
        .ok()
        .and_then(|output| parse_active_security(&String::from_utf8_lossy(&output.stdout)));

    Some(WifiInfo {
        ssid,
        interface: interface.to_owned(),
        signal_dbm: link.signal_dbm,
        band: link.freq_mhz.map(band_label),
        channel: link.freq_mhz.and_then(channel_for_freq),
        security,
    })
}

pub fn read_ipv4(interface: &str) -> Option<String> {
    let output = Command::new("ip")
        .args(["-o", "-4", "addr", "show", "dev", interface])
        .output()
        .ok()?;

    parse_ipv4_with_prefix(&String::from_utf8_lossy(&output.stdout))
}

pub fn read_nameservers() -> Vec<String> {
    fs::read_to_string("/etc/resolv.conf")
        .map(|content| parse_nameservers(&content))
        .unwrap_or_default()
}

/// Un ping bloqueante de 1 segundo máximo; `None` = perdido.
pub fn ping_once(target: &str) -> Option<f32> {
    let output = Command::new("ping").args(["-n", "-c", "1", "-W", "1", target]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    parse_ping_ms(&String::from_utf8_lossy(&output.stdout))
}
