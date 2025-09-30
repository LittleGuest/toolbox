use std::collections::HashMap;

use battery::Battery;
use serde::Serialize;
use sysinfo::{Component, Disk, Process, System};

#[derive(Serialize, Default)]
pub struct SysMonitorData {
    pub host: HostData,
    pub disks: Vec<DiskData>,
    pub sensors: HashMap<String, f32>,
    pub load_avg: f64,
}

#[derive(Serialize, Default)]
pub struct ProcessData {
    name: String,
    pub memory: f64,
    pid: String,
}

impl ProcessData {
    pub fn new(process: &Process) -> Self {
        Self {
            name: process.name().to_string_lossy().to_string(),
            memory: MemoryData::format_memory(process.memory()),
            pid: process.pid().to_string(),
        }
    }
}

#[derive(Serialize, Default)]
pub struct SensorData {
    label: String,
    temperature: f32,
}

impl SensorData {
    pub fn new(component: &Component) -> Self {
        Self {
            label: component.label().to_string(),
            temperature: component.temperature().unwrap_or_default(),
        }
    }
}

#[derive(Serialize, Default)]
pub struct CpuData {
    chip_name: String,
    physical_core_count: usize,
    global_usage: f32,
    cores: Vec<CpuCoreData>,
}

impl CpuData {
    pub fn new(sysinfo: &System, cores: Vec<CpuCoreData>) -> Self {
        Self {
            chip_name: sysinfo.cpus()[0].brand().to_string(),
            physical_core_count: System::physical_core_count().unwrap_or_default(),
            global_usage: sysinfo.global_cpu_usage(),
            cores,
        }
    }
}

#[derive(Serialize, Default)]
pub struct CpuCoreData {
    usage: f32,
    frequency: u64,
}

impl CpuCoreData {
    pub fn new(usage: f32, frequency: u64) -> Self {
        Self { usage, frequency }
    }
}

#[derive(Serialize, Default)]
pub struct BatteryData {
    temperature: String,
    cycle_count: u32,
    // 充电状态
    state: i32,
    // 电量百分比
    percentage: f32,
    // // 还需多久充满
    // time_to_full: u32,
    // // 电池剩余使用时间
    // time_to_empty: u32,
    // 电池健康
    state_of_health: String,
}

impl BatteryData {
    pub fn new(battery: &Battery) -> Self {
        Self {
            temperature: format!("{:.2}℃", battery.temperature().unwrap().value - 273.15),
            cycle_count: battery.cycle_count().unwrap_or(0),
            state: match battery.state() {
                battery::State::Full => 1,
                battery::State::Charging => 2,
                battery::State::Discharging => 3,
                battery::State::Empty => 0,
                _ => -1,
            },
            percentage: battery.state_of_charge().value * 100.,
            state_of_health: format!("{:.2}%", battery.state_of_health().value * 100.),
        }
    }
}

#[derive(Serialize, Default)]
pub struct HostData {}

#[derive(Serialize, Default)]
pub struct MemoryData {
    total_memory: f64,
    total_swap: f64,
    used_memory: f64,
    used_swap: f64,
}

impl MemoryData {
    pub fn new(sysinfo: &System) -> Self {
        Self {
            total_memory: MemoryData::format_memory(sysinfo.total_memory()),
            total_swap: MemoryData::format_memory(sysinfo.total_swap()),
            used_memory: MemoryData::format_memory(sysinfo.used_memory()),
            used_swap: MemoryData::format_memory(sysinfo.used_swap()),
        }
    }

    fn format_memory(bytes: u64) -> f64 {
        return bytes as f64 / (1024. * 1024. * 1024.);
    }
}

#[derive(Serialize, Default)]
pub struct DiskData {
    name: String,
}

impl DiskData {
    pub fn new(disk: &Disk) -> Self {
        Self {
            name: format!("{:?}", disk),
        }
    }
}
