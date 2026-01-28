use common::entity::hardware::CPU;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

const CPUINFO_FILE: &str = "/proc/cpuinfo";

pub fn get_cpu_info() -> CPU {
    let mut cpu = CPU {
        vendor_id: String::new(),
        model_name: String::new(),
        speed: 0,
        cores: 0,
        threads: 0,
        cpus: 0,
        flags: Vec::new(),
    };

    let file = match File::open(CPUINFO_FILE) {
        Ok(f) => f,
        Err(e) => {
            // In a real scenario, consider logging this error and returning a Result
            // For now, maintaining panic behavior consistent with previous unwrap_or_else
            eprintln!("Failed to open {}: {}", CPUINFO_FILE, e); // Log to stderr
            return cpu; // Return default/empty CPU struct on error
        }
    };
    let reader = BufReader::new(file);

    let mut cores_per_physical_cpu: u32 = 0;
    let mut physical_ids_set: HashSet<String> = HashSet::new();
    let mut logical_processor_count: u32 = 0;

    let mut vendor_id_found = false;
    let mut model_name_found = false;
    let mut speed_found = false;
    let mut flags_found = false;
    let mut cores_per_cpu_parsed = false; // To ensure "cpu cores" is taken once effectively

    let cpu_speed = get_cpu_speed();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue, // Skip bad lines
        };

        let parts: Vec<&str> = line.splitn(2, ':').map(str::trim).collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0];
        let value = parts[1];

        match key {
            "vendor_id" if !vendor_id_found => {
                cpu.vendor_id = value.to_string();
                vendor_id_found = true;
            }
            "model name" if !model_name_found => {
                cpu.model_name = value.to_string();
                model_name_found = true;
            }
            "cpu MHz" if !speed_found => {
                if let Ok(s) = value.parse::<f32>() {
                    if cpu_speed > 0 {
                        cpu.speed = cpu_speed;
                    } else {
                        cpu.speed = s as u32;
                    }
                    speed_found = true;
                }
            }
            "flags" if !flags_found => {
                cpu.flags = value
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                flags_found = true;
            }
            "cpu cores" if !cores_per_cpu_parsed => {
                if let Ok(c) = value.parse::<u32>() {
                    if c > 0 {
                        // Only accept a valid, positive number of cores
                        cores_per_physical_cpu = c;
                        cores_per_cpu_parsed = true;
                    }
                }
            }
            "physical id" => {
                physical_ids_set.insert(value.to_string());
            }
            "processor" => {
                logical_processor_count += 1;
            }
            _ => {}
        }
    }

    // If after parsing the whole file, essential information is missing,
    // we might have a very minimal or strange /proc/cpuinfo.
    if logical_processor_count == 0 {
        // No processors found, CPU info is likely invalid or system is unusual.
        // Return the mostly empty CPU struct.
        // Set cpus to 1 to avoid division by zero or illogical states if other fields were partially filled.
        cpu.cpus = cpu.cpus.max(1);
        cpu.cores = cpu.cores.max(1);
        cpu.threads = cpu.threads.max(1);
        return cpu;
    }

    cpu.threads = logical_processor_count;

    let num_physical_sockets = if physical_ids_set.is_empty() {
        1 // Assume at least one socket if no physical_id entries but processors exist
    } else {
        physical_ids_set.len() as u32
    };
    cpu.cpus = num_physical_sockets;

    if cores_per_physical_cpu > 0 {
        cpu.cores = cores_per_physical_cpu * cpu.cpus;
    } else {
        // Fallback: if "cpu cores" is not found or 0,
        // use total logical processors as a best guess for total cores.
        cpu.cores = logical_processor_count;
    }

    // Final sanity checks to ensure logical consistency
    cpu.cpus = cpu.cpus.max(1); // Should be at least one physical CPU package.
    cpu.cores = cpu.cores.max(cpu.cpus); // Total cores must be at least the number of CPU packages.
    cpu.cores = cpu.cores.max(1); // At least one core.
    cpu.threads = cpu.threads.max(cpu.cores); // Logical threads must be >= total physical cores.
    cpu.threads = cpu.threads.max(1); // At least one thread.

    cpu
}
// get cpu speed by /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq if cpu enable cpufreq
fn get_cpu_speed() -> u32 {
    println!("test to get cpu by cpuinfo_max_freq");
    let file = match File::open("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq") {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() {
        if let Ok(speed) = line.trim().parse::<u32>() {
            println!("cpu speed: {}", speed);
            return speed / 1000;
        } else {
            return 0;
        }
    }
    0
}
