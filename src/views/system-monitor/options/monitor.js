export class SysMonitorData {
    sensors;
    load_avg;
}

export class CpuData {
    chip_name;
    physical_core_count;
    global_usage;
    cores;
}

export class CpuCoreData {
    usage;
    frequency;
}

export class ProcessData {
    memory;
    name;
    pid;
}

export class SensorData {
    label;
    temperature;
}

export class BatteryData {
    
    // 电池温度
    temperature;
    // 循环周期
    cycle_count;
    // 充电状态
    state;
    // 电量百分比
    percentage;
    // // 还需多久充满
    // time_to_full: u32,
    // // 电池剩余使用时间
    // time_to_empty: u32,
    // 电池健康
    state_of_health;
}

export class MemoryData {
    total_memory;
    total_swap;
    used_memory;
    used_swap;
}