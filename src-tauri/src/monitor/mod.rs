use std::collections::HashMap;

use battery::Manager;
use monitor::{
    BatteryData, CpuCoreData, CpuData, HostData, MemoryData, ProcessData, SysMonitorData,
};
use sysinfo::{Components, ProcessesToUpdate, System};

mod monitor;

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
    let mut sys = System::new_all();
    sys.refresh_memory();
    return MemoryData::new(&sys);
}

#[tauri::command]
pub fn monitor_cpu_info() -> CpuData {
    let mut sys = System::new_all();
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
pub fn monitor_process_info() -> Vec<ProcessData> {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
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
