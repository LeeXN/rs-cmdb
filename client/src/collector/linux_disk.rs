use common::entity::hardware::{Disk, Partition, StorageType};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

// Helper function to format size in bytes to appropriate unit and return formatted string
fn format_storage_size(bytes: u64) -> (String, String) {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f = bytes as f64;

    if bytes_f >= GB {
        (format!("{:.1}", bytes_f / GB), "GB".to_string())
    } else if bytes_f >= MB {
        (format!("{:.1}", bytes_f / MB), "MB".to_string())
    } else if bytes_f >= KB {
        (format!("{:.1}", bytes_f / KB), "KB".to_string())
    } else {
        (format!("{}", bytes), "B".to_string()) // Bytes as integer
    }
}

fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path).map(|s| s.trim().to_string())
}

// Check if lsblk command is available in PATH
fn has_lsblk() -> bool {
    Command::new("which")
        .arg("lsblk")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Attempt to get firmware revision and serial number from sysfs
fn get_firmware_serial_from_sysfs(base_path: &str) -> (String, String) {
    let serial_path = format!("{}/device/serial", base_path);
    let serial_number = read_to_string(serial_path).unwrap_or_else(|_| "Unknown".to_string());

    let rev_path = format!("{}/device/rev", base_path);
    let mut firmware_version = read_to_string(rev_path).unwrap_or_else(|_| "Unknown".to_string());

    // Fallback to firmware_rev if rev is not available
    if firmware_version == "Unknown" {
        let firmware_rev_path = format!("{}/device/firmware_rev", base_path);
        firmware_version =
            read_to_string(firmware_rev_path).unwrap_or_else(|_| "Unknown".to_string());
    }

    (firmware_version, serial_number)
}

// Get info using lsblk command
fn get_lsblk_info() -> HashMap<String, (String, String)> {
    let mut info_map = HashMap::new();
    let output = Command::new("lsblk")
        .args(["-n", "-o", "NAME,REV,SERIAL", "--raw"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let name = parts[0].to_string();
                    // REV and SERIAL might be missing or empty
                    let rev = parts
                        .get(1)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let serial = parts
                        .get(2)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    info_map.insert(name, (rev, serial));
                }
            }
        } else {
            eprintln!(
                "lsblk command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        eprintln!("Failed to execute lsblk command");
    }

    info_map
}

fn detect_storage_type(device: &str) -> StorageType {
    if device.starts_with("nvme") {
        StorageType::NVMe
    } else if let Ok(rotational) = read_to_string(format!("/sys/block/{}/queue/rotational", device))
    {
        match rotational.as_str() {
            "0" => StorageType::SSD,
            "1" => StorageType::HDD,
            _ => StorageType::Unknown,
        }
    } else {
        StorageType::Unknown
    }
}

fn get_partitions(device: &str) -> Vec<Partition> {
    let mut partitions = Vec::new();
    let base_path = PathBuf::from(format!("/sys/block/{}", device));

    if let Ok(entries) = fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            let prefix = if device.starts_with("nvme") {
                format!("{}p", device)
            } else {
                device.to_string()
            };

            if filename.starts_with(&prefix) && filename != device {
                // 获取大小
                let size_path = path.join("size");
                if let Ok(size_str) = read_to_string(&size_path) {
                    if let Ok(size_sectors) = size_str.parse::<u64>() {
                        let size_bytes = size_sectors * 512;
                        let (formatted_size_str, unit) = format_storage_size(size_bytes);
                        partitions.push(Partition {
                            name: format!("/dev/{}", filename),
                            size: formatted_size_str,
                            size_unit: unit,
                        });
                    }
                }
            }
        }
    }

    partitions
}

pub fn collect_disks() -> Vec<Disk> {
    let mut disks = Vec::new();
    let sys_block = Path::new("/sys/block");

    let lsblk_available = has_lsblk();
    let lsblk_info = if lsblk_available {
        get_lsblk_info()
    } else {
        HashMap::new() // Empty map if lsblk is not available
    };

    if let Ok(entries) = fs::read_dir(sys_block) {
        for entry in entries.flatten() {
            let device_name = entry.file_name().to_string_lossy().to_string();

            // Skip virtual devices
            if device_name.starts_with("loop") || device_name.starts_with("ram") {
                continue;
            }

            let base_path = format!("/sys/block/{}", device_name);

            let vendor = read_to_string(format!("{}/device/vendor", base_path))
                .unwrap_or_else(|_| "Unknown".into());
            let model = read_to_string(format!("{}/device/model", base_path))
                .unwrap_or_else(|_| "Unknown".into());

            let size_sectors: u64 = read_to_string(format!("{}/size", base_path))
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);

            let size_bytes = size_sectors * 512;
            let (formatted_size_str, unit) = format_storage_size(size_bytes);

            // Determine firmware version and serial number using the hybrid approach
            let (mut final_firmware, mut final_serial) =
                ("Unknown".to_string(), "Unknown".to_string());

            if lsblk_available {
                if let Some((lsblk_fw, lsblk_sn)) = lsblk_info.get(&device_name) {
                    final_firmware = lsblk_fw.clone();
                    final_serial = lsblk_sn.clone();
                }
            }

            // If lsblk didn't provide info (or wasn't available), try sysfs
            if final_firmware == "Unknown" || final_serial == "Unknown" {
                let (sysfs_fw, sysfs_sn) = get_firmware_serial_from_sysfs(&base_path);
                if final_firmware == "Unknown" {
                    final_firmware = sysfs_fw;
                }
                if final_serial == "Unknown" {
                    final_serial = sysfs_sn;
                }
            }

            let storage_type = detect_storage_type(&device_name);
            let partitions = get_partitions(&device_name);

            disks.push(Disk {
                vendor,
                size: formatted_size_str,
                size_unit: unit,
                model,
                storage_type,
                firmware_version: final_firmware, // Use the determined value
                serial_number: final_serial,      // Use the determined value
                parted: !partitions.is_empty(),
                partitions,
            });
        }
    }

    disks
}
