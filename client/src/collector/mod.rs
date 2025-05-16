pub mod linux_collector;
pub mod linux_cpu;
pub mod linux_disk;
pub mod linux_nic;
pub mod linux_gpu;
pub mod linux_ram;
pub mod linux_ipmi;

use common::entity::hardware::Hardware;

pub fn collect_all_hardware_info() -> Hardware {
    linux_collector::collect_hardware()
}