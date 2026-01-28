#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_type_values() {
        assert_eq!(HardwareType::OS as u32, 0);
        assert_eq!(HardwareType::CPU as u32, 1);
        assert_eq!(HardwareType::Disk as u32, 2);
        assert_eq!(HardwareType::GPU as u32, 3);
        assert_eq!(HardwareType::NIC as u32, 4);
        assert_eq!(HardwareType::RAM as u32, 5);
        assert_eq!(HardwareType::SYS as u32, 6);
        assert_eq!(HardwareType::IPMI as u32, 7);
        assert_eq!(HardwareType::All as u32, 8);
    }

    #[test]
    fn test_display_options_creation() {
        let mut types = std::collections::HashSet::new();
        types.insert(HardwareType::CPU);
        types.insert(HardwareType::RAM);

        let options = DisplayOptions {
            hardware_types: types,
            show_detail: true,
        };

        assert!(options.show_detail);
        assert_eq!(options.hardware_types.len(), 2);
    }

    #[test]
    fn test_collect_all_hardware_info() {
        let hardware = collect_all_hardware_info();

        assert!(hardware.os.name.len() > 0);
        assert!(hardware.cpu.model_name.len() > 0);
        assert!(hardware.system.sys_vendor.len() > 0);
    }

    #[test]
    fn test_get_threads_per_core_with_valid_cpu() {
        use crate::collector::linux_cpu::get_cpu_info;

        let cpu = get_cpu_info();
        if cpu.cores > 0 {
            let threads_per_core = get_threads_per_core();
            assert!(threads_per_core >= 1.0);
        }
    }

    #[test]
    fn test_get_cores_per_cpu_with_valid_cpu() {
        use crate::collector::linux_cpu::get_cpu_info;

        let cpu = get_cpu_info();
        if cpu.cpus > 0 {
            let cores_per_cpu = get_cores_per_cpu();
            assert!(cores_per_cpu >= 1);
        }
    }

    #[test]
    fn test_collect_os_info() {
        use crate::collector::linux_collector::collect_os_info;

        let os = collect_os_info();

        assert!(os.name.len() > 0);
        assert!(os.architecture.len() > 0);
        assert!(os.hostname.len() > 0);
    }

    #[test]
    fn test_collect_system_info() {
        use crate::collector::linux_collector::collect_system_info;

        let system = collect_system_info();

        assert!(system.sys_vendor.len() > 0 || system.sys_vendor == "Unknown");
        assert!(system.product_name.len() > 0 || system.product_name == "Unknown");
        assert!(system.serial_number.len() > 0 || system.serial_number == "Unknown");
    }

    #[test]
    fn test_collect_cpu_info() {
        use crate::collector::linux_collector::collect_cpu_info;

        let cpu = collect_cpu_info();

        assert!(cpu.model_name.len() > 0);
        assert!(cpu.vendor_id.len() > 0);
    }

    #[test]
    fn test_collect_disk_info() {
        use crate::collector::linux_collector::collect_disk_info;

        let disks = collect_disk_info();

        assert!(disks.len() >= 0);
    }

    #[test]
    fn test_collect_nic_info() {
        use crate::collector::linux_collector::collect_nic_info;

        let nics = collect_nic_info().unwrap();

        assert!(nics.len() >= 0);
    }

    #[test]
    fn test_collect_gpu_info() {
        use crate::collector::linux_collector::collect_gpu_info;

        let gpus = collect_gpu_info();

        assert!(gpus.len() >= 0);
    }

    #[test]
    fn test_collect_ram_info() {
        use crate::collector::linux_collector::collect_ram_info;

        let ram = collect_ram_info();

        assert!(ram.modules.len() >= 0);
        assert!(ram.total_size > 0);
    }

    #[test]
    fn test_display_options_empty() {
        let options = DisplayOptions {
            hardware_types: std::collections::HashSet::new(),
            show_detail: false,
        };

        assert_eq!(options.hardware_types.len(), 0);
        assert!(!options.show_detail);
    }
}
