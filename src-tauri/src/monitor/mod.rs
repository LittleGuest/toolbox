use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use battery::{Battery, Manager};
use serde::Serialize;
use sysinfo::{
    Component, Components, Disk, Disks, Process, ProcessRefreshKind, ProcessesToUpdate, System,
};

#[derive(Serialize, Default)]
pub struct SysMonitorData {
    pub host: HostData,
    pub disks: Vec<DiskData>,
    pub sensors: HashMap<String, f32>,
    pub load_avg: f64,
}

#[derive(Serialize, Default)]
pub struct HostData {
    pub hostname: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_brand: String,
}

#[derive(Serialize, Default)]
pub struct CpuData {
    pub chip_name: String,
    pub physical_core_count: usize,
    pub global_usage: f32,
    pub cores: Vec<CpuCoreData>,
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
    pub usage: f32,
    pub frequency: u64,
}

impl CpuCoreData {
    pub fn new(usage: f32, frequency: u64) -> Self {
        Self { usage, frequency }
    }
}

#[derive(Serialize, Default)]
pub struct MemoryData {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
}

impl MemoryData {
    pub fn new(sysinfo: &System) -> Self {
        Self {
            total_memory: sysinfo.total_memory(),
            used_memory: sysinfo.used_memory(),
            total_swap: sysinfo.total_swap(),
            used_swap: sysinfo.used_swap(),
        }
    }
}

#[derive(Serialize, Default)]
pub struct DiskData {
    pub name: String,
    pub file_system: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub usage: DiskUsage,
}

#[derive(Serialize, Default)]
pub struct DiskUsage {
    pub total_written_bytes: u64,
    pub written_bytes: u64,
    pub total_read_bytes: u64,
    pub read_bytes: u64,
}

impl DiskData {
    pub fn new(disk: &Disk) -> Self {
        Self {
            name: format!("{:?}", disk),
            file_system: disk.file_system().to_string_lossy().to_string(),
            mount_point: disk.mount_point().display().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            usage: DiskUsage {
                total_written_bytes: disk.usage().total_written_bytes,
                written_bytes: disk.usage().written_bytes,
                total_read_bytes: disk.usage().total_read_bytes,
                read_bytes: disk.usage().read_bytes,
            },
        }
    }
}

#[derive(Serialize, Default)]
pub struct ProcessData {
    pub pid: String,
    pub name: String,
    pub cpu: f32,
    pub memory: u64,
}

impl ProcessData {
    pub fn new(process: &Process) -> Self {
        Self {
            pid: process.pid().to_string(),
            name: process.name().to_string_lossy().to_string(),
            cpu: process.cpu_usage(),
            memory: process.memory(),
        }
    }
}

#[derive(Serialize, Default)]
pub struct SensorData {
    pub label: String,
    pub temperature: f32,
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
pub struct BatteryData {
    pub temperature: String,
    pub cycle_count: u32,
    // 充电状态
    pub state: i32,
    // 电量百分比
    pub percentage: f32,
    // // 还需多久充满
    // pub time_to_full: u32,
    // // 电池剩余使用时间
    // pub time_to_empty: u32,
    // 电池健康
    pub state_of_health: String,
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

static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();

fn get_system() -> &'static Mutex<System> {
    SYSTEM.get_or_init(|| Mutex::new(System::new_all()))
}

#[tauri::command]
pub fn monitor_host_info() -> HostData {
    let mut sys = get_system().lock().unwrap();
    sys.refresh_all();
    return HostData {
        hostname: System::host_name().unwrap().to_string(),
        os_version: System::os_version().unwrap().to_string(),
        kernel_version: System::kernel_version().unwrap().to_string(),
        cpu_brand: sys.cpus()[0].brand().to_string(),
    };
}

#[tauri::command]
pub fn monitor_system_info() -> SysMonitorData {
    let components = Components::new_with_refreshed_list();
    let mut sensors = HashMap::new();
    for component in &components {
        sensors.insert(
            component.label().to_string(),
            component.temperature().unwrap_or_default(),
        );
    }

    return SysMonitorData {
        host: HostData::default(),
        disks: vec![],
        sensors,
        load_avg: System::load_average().one,
    };
}

#[tauri::command]
pub fn monitor_memory_info() -> MemoryData {
    let mut sys = get_system().lock().unwrap();
    sys.refresh_memory();
    return MemoryData::new(&sys);
}

#[tauri::command]
pub fn monitor_cpu_info() -> CpuData {
    let mut sys = get_system().lock().unwrap();
    sys.refresh_cpu_all();
    let mut cpu_cores = vec![];
    let cpus = sys.cpus();
    for cpu in cpus.iter() {
        cpu_cores.push(CpuCoreData::new(cpu.cpu_usage(), cpu.frequency()));
    }
    let cpu = CpuData::new(&sys, cpu_cores);
    return cpu;
}

#[tauri::command]
pub fn monitor_disk_info() -> Vec<DiskData> {
    let disks = Disks::new_with_refreshed_list();
    let mut disk_data = vec![];
    for disk in disks.list() {
        disk_data.push(DiskData::new(disk));
    }
    return disk_data;
}

#[tauri::command]
pub fn monitor_process_info() -> Vec<ProcessData> {
    let mut sys = get_system().lock().unwrap();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu(),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    let mut processes = vec![];
    for (pid, process) in sys.processes() {
        processes.push(ProcessData::new(&process));
    }
    processes.sort_by(|a, b| b.memory.partial_cmp(&a.memory).unwrap());
    return processes;
}

#[tauri::command]
pub fn monitor_battery_info() -> BatteryData {
    let manager = Manager::new().unwrap();
    let mut batteries = vec![];
    for (_, battery) in manager.batteries().unwrap().enumerate() {
        batteries.push(BatteryData::new(&battery.unwrap()));
        break;
    }
    if batteries.len() == 0 {
        return BatteryData::default();
    } else {
        return batteries.pop().unwrap();
    }
}

#[tauri::command]
pub fn kill_process(pid: u32) -> Result<(), String> {
    let mut sys = get_system().lock().unwrap();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
        if process.kill() {
            Ok(())
        } else {
            Err(format!("Failed to kill process {}", pid))
        }
    } else {
        Err(format!("Process {} not found", pid))
    }
}
