use serde::Serialize;

#[tauri::command]
pub fn monitor_host_info() -> impl Serialize {
    monitor::host_info()
}

#[tauri::command]
pub fn monitor_system_info() -> impl Serialize {
    monitor::system_info()
}

#[tauri::command]
pub fn monitor_memory_info() -> impl Serialize {
    monitor::memory_info()
}

#[tauri::command]
pub fn monitor_cpu_info() -> impl Serialize {
    monitor::cpu_info()
}

#[tauri::command]
pub fn monitor_disk_info() -> impl Serialize {
    monitor::disk_info()
}

#[tauri::command]
pub fn monitor_process_info() -> impl Serialize {
    monitor::process_info()
}

#[tauri::command]
pub fn monitor_battery_info() -> impl Serialize {
    monitor::battery_info()
}

#[tauri::command]
pub fn monitor_kill_process(pid: u32) -> Result<(), String> {
    monitor::kill_process(pid)
}
