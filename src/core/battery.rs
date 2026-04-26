use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::core::config::CONFIG;

// Conversion factor: voltage (μV) × current (μA) = power (μW²)? — actually μV × μA = μW × 10⁻⁶,
// so dividing the product by 10¹² yields W. Same factor for charge × voltage: μAh × μV / 10¹² = Wh.
const MICRO_PRODUCT_TO_BASE: f64 = 1_000_000_000_000.0;
// Conversion factor for raw μW values reported in `power_now`.
const MICRO_TO_BASE: f64 = 1_000_000.0;
const MINUTES_PER_HOUR: f64 = 60.0;
const MINUTES_PER_DAY: f64 = 1440.0;

#[derive(Debug, PartialEq, Clone)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BatteryInfo {
    pub capacity: u8,
    pub status: BatteryStatus,
    pub power_w: Option<f64>,
    pub time_remaining_min: Option<f64>,
}

fn resolved_device() -> Option<&'static str> {
    static DEVICE: OnceLock<Option<String>> = OnceLock::new();
    DEVICE
        .get_or_init(|| {
            let configured = &CONFIG.battery.device;
            if device_exists(configured) {
                return Some(configured.clone());
            }
            log::warn!(
                "Configured battery device '{}' not found, falling back to auto-detect",
                configured
            );
            autodetect_battery()
        })
        .as_deref()
}

fn device_exists(name: &str) -> bool {
    PathBuf::from("/sys/class/power_supply").join(name).is_dir()
}

fn autodetect_battery() -> Option<String> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let type_path = entry.path().join("type");
        if let Ok(t) = fs::read_to_string(&type_path)
            && t.trim() == "Battery"
        {
            log::info!("Auto-detected battery device: {}", name);
            return Some(name);
        }
    }
    log::warn!("No battery device found in /sys/class/power_supply");
    None
}

fn read_sysfs(file: &str) -> Option<String> {
    let device = resolved_device()?;
    fs::read_to_string(format!("/sys/class/power_supply/{device}/{file}"))
        .ok()
        .map(|s| s.trim().to_string())
}

fn read_sysfs_f64(file: &str) -> Option<f64> {
    read_sysfs(file)?.parse().ok()
}

pub fn get_battery_info() -> Option<BatteryInfo> {
    let capacity: u8 = read_sysfs("capacity")?.parse().ok()?;

    let status_str = read_sysfs("status").unwrap_or_default();
    let status = match status_str.as_str() {
        "Charging" => BatteryStatus::Charging,
        "Discharging" => BatteryStatus::Discharging,
        "Full" => BatteryStatus::Full,
        _ => BatteryStatus::Unknown,
    };

    let power_w = compute_power_w();
    let time_remaining_min = power_w.and_then(|p| compute_time_remaining_min(&status, p));

    Some(BatteryInfo {
        capacity,
        status,
        power_w,
        time_remaining_min,
    })
}

fn compute_power_w() -> Option<f64> {
    if let Some(power_now) = read_sysfs_f64("power_now") {
        let w = power_now / MICRO_TO_BASE;
        if w > 0.0 {
            return Some(w);
        }
    }
    if let (Some(current_now), Some(voltage_now)) =
        (read_sysfs_f64("current_now"), read_sysfs_f64("voltage_now"))
    {
        let w = (current_now * voltage_now) / MICRO_PRODUCT_TO_BASE;
        if w > 0.0 {
            return Some(w);
        }
    }
    None
}

fn compute_time_remaining_min(status: &BatteryStatus, power_w: f64) -> Option<f64> {
    if let (Some(energy_now), Some(energy_full)) =
        (read_sysfs_f64("energy_now"), read_sysfs_f64("energy_full"))
    {
        let energy_target = if *status == BatteryStatus::Charging {
            f64::max(0.0, energy_full - energy_now)
        } else {
            energy_now
        };

        let energy_target_wh = energy_target / MICRO_TO_BASE;
        return Some((energy_target_wh / power_w) * MINUTES_PER_HOUR);
    }

    if let (Some(charge_now), Some(charge_full), Some(voltage_now)) = (
        read_sysfs_f64("charge_now"),
        read_sysfs_f64("charge_full"),
        read_sysfs_f64("voltage_now"),
    ) {
        let charge_target = if *status == BatteryStatus::Charging {
            f64::max(0.0, charge_full - charge_now)
        } else {
            charge_now
        };

        let energy_target_wh = (charge_target * voltage_now) / MICRO_PRODUCT_TO_BASE;
        return Some((energy_target_wh / power_w) * MINUTES_PER_HOUR);
    }

    None
}

pub fn time_remaining_is_meaningful(min: f64) -> bool {
    min > 0.0 && min < MINUTES_PER_DAY
}
