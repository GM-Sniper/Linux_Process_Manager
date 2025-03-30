use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use std::collections::HashMap;

pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
    pub status: String,
}

pub struct ProcessManager {
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all(); 
        system.refresh_all(); 
        ProcessManager { system }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        let mut processes = Vec::new(); //
        
        for (pid, process) in self.system.processes() {
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                parent_pid: process.parent().map(|p| p.as_u32()),
                start_time: process.start_time(),
                status: process.status().to_string(),
                user: process.user_id(),
            });
        }
        
        // Sort by CPU usage (descending)
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        
        processes
    }
}
