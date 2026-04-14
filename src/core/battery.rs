use std::fs;

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
    pub power_w: f64,
    pub time_remaining_min: f64,
}

fn read_sysfs(file: &str) -> Option<String> {
    let device = &crate::core::config::CONFIG.get()?.battery.device;

    fs::read_to_string(format!("/sys/class/power_supply/{}/{}", device, file))
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

    let mut power_w = 0.0;

    if let Some(power_now) = read_sysfs_f64("power_now") {
        power_w = power_now / 1_000_000.0;
    } else if let (Some(current_now), Some(voltage_now)) =
        (read_sysfs_f64("current_now"), read_sysfs_f64("voltage_now"))
    {
        power_w = (current_now * voltage_now) / 1_000_000_000_000.0;
    }

    let mut time_remaining_min = 0.0;
    if power_w > 0.0 {
        if let (Some(energy_now), Some(energy_full)) =
            (read_sysfs_f64("energy_now"), read_sysfs_f64("energy_full"))
        {
            let energy_target = if status == BatteryStatus::Charging {
                f64::max(0.0, energy_full - energy_now)
            } else {
                energy_now
            };

            let energy_target_wh = energy_target / 1_000_000.0;
            time_remaining_min = (energy_target_wh / power_w) * 60.0;
        } else if let (Some(charge_now), Some(charge_full), Some(voltage_now)) = (
            read_sysfs_f64("charge_now"),
            read_sysfs_f64("charge_full"),
            read_sysfs_f64("voltage_now"),
        ) {
            let charge_target = if status == BatteryStatus::Charging {
                f64::max(0.0, charge_full - charge_now)
            } else {
                charge_now
            };

            let energy_target_wh = (charge_target * voltage_now) / 1_000_000_000_000.0;
            time_remaining_min = (energy_target_wh / power_w) * 60.0;
        }
    }

    Some(BatteryInfo {
        capacity,
        status,
        power_w,
        time_remaining_min,
    })
}
