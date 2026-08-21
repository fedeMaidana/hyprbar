// ─── < Imports > ────────────────────────────────────────────────────

use std::fs;
use std::path::{Path, PathBuf};

use super::state::BatteryData;

// ─── < Constants > ────────────────────────────────────────────────────

const POWER_SUPPLY_DIR: &str = "/sys/class/power_supply";

const MICRO: f32 = 1_000_000.0;

// ─── < Public Functions > ────────────────────────────────────────────────────

/// Directorio de la primera batería (type == "Battery").
pub fn find_battery_dir() -> Option<PathBuf> {
    find_supply_of_type("Battery")
}

/// Directorio del adaptador de corriente (type == "Mains").
pub fn find_adapter_dir() -> Option<PathBuf> {
    find_supply_of_type("Mains")
}

/// Lee la batería completa. La historia de consumo la mantiene el worker;
/// acá `power_history` queda vacío.
pub fn read_battery(battery_dir: &Path, adapter_dir: Option<&Path>) -> BatteryData {
    let voltage_v = read_scaled(battery_dir, "voltage_now");

    // Los drivers exponen energía (µWh) o carga (µAh); la carga se pasa
    // a Wh con el voltaje para que el panel hable siempre en Wh.
    let charge_wh = read_scaled(battery_dir, "energy_now").or_else(|| ah_to_wh(read_scaled(battery_dir, "charge_now"), voltage_v));
    let charge_full_wh = read_scaled(battery_dir, "energy_full").or_else(|| ah_to_wh(read_scaled(battery_dir, "charge_full"), voltage_v));
    let design_wh =
        read_scaled(battery_dir, "energy_full_design").or_else(|| ah_to_wh(read_scaled(battery_dir, "charge_full_design"), voltage_v));

    let power_w = read_scaled(battery_dir, "power_now")
        .or_else(|| match (read_scaled(battery_dir, "current_now"), voltage_v) {
            (Some(amps), Some(volts)) => Some(amps * volts),
            _ => None,
        })
        .map(f32::abs);

    let status = read_string(battery_dir, "status").map(|value| value.to_lowercase());

    let minutes_left = match (status.as_deref(), charge_wh, charge_full_wh, power_w) {
        (Some("discharging"), Some(now), _, Some(watts)) if watts > 0.1 => Some((now / watts * 60.0) as u32),
        (Some("charging"), Some(now), Some(full), Some(watts)) if watts > 0.1 => Some(((full - now).max(0.0) / watts * 60.0) as u32),
        _ => None,
    };

    let health_percent = match (charge_full_wh, design_wh) {
        (Some(full), Some(design)) if design > 0.0 => Some(((full / design) * 100.0).clamp(0.0, 100.0) as u8),
        _ => None,
    };

    BatteryData {
        percent: read_string(battery_dir, "capacity").and_then(|value| value.parse().ok()),
        status,
        minutes_left,
        power_w,
        power_history: Default::default(),
        charge_wh,
        charge_full_wh,
        voltage_v,
        cycles: read_string(battery_dir, "cycle_count").and_then(|value| value.parse().ok()),
        health_percent,
        technology: read_string(battery_dir, "technology").map(|value| value.to_lowercase()),
        name: battery_dir.file_name().map(|name| name.to_string_lossy().into_owned()),
        // sysfs la expone en décimas de grado.
        cell_temp_c: read_string(battery_dir, "temp")
            .and_then(|value| value.parse::<f32>().ok())
            .map(|tenths| tenths / 10.0),
        adapter_online: adapter_dir.and_then(|dir| read_string(dir, "online")).map(|value| value == "1"),
    }
}

// ─── < Private Functions > ────────────────────────────────────────────────────

fn find_supply_of_type(wanted: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(POWER_SUPPLY_DIR).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        let Ok(kind) = fs::read_to_string(path.join("type")) else {
            continue;
        };

        if kind.trim() == wanted {
            return Some(path);
        }
    }

    None
}

fn read_string(dir: &Path, file: &str) -> Option<String> {
    fs::read_to_string(dir.join(file)).ok().map(|value| value.trim().to_owned())
}

/// Valores µ-escala de sysfs (µWh, µW, µV, µAh) a unidades enteras.
fn read_scaled(dir: &Path, file: &str) -> Option<f32> {
    read_string(dir, file)?.parse::<f32>().ok().map(|micro| micro / MICRO)
}

fn ah_to_wh(amp_hours: Option<f32>, volts: Option<f32>) -> Option<f32> {
    match (amp_hours, volts) {
        (Some(ah), Some(v)) => Some(ah * v),
        _ => None,
    }
}
