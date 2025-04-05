use sysinfo::{ProcessExt, System, SystemExt, PidExt, UserExt};
use procfs::process::Process as ProcfsProcess; // Import procfs for nice value
use std::convert::TryInto; // Import the try_into function
use chrono::{DateTime, Local, TimeZone};

#[derive(Clone)] 
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
    pub status: String,
    pub user: Option<String>,
    pub nice: i32, 
    pub startTime: String,
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
        let mut processes = Vec::new();
        
        for (pid, process) in self.system.processes() {
            // Convert pid to i32 for ProcfsProcess::new()
            let pid_i32: i32 = pid.as_u32().try_into().unwrap_or(0); // Safe conversion

            // Retrieve nice value using procfs
            let nice_value = ProcfsProcess::new(pid_i32)
                .and_then(|p| p.stat().map(|stat| stat.nice))
                .unwrap_or(0); // Default to 0 if retrieval fails
            // Format the start time
            let formatted_time = format_timestamp(process.start_time());
            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                parent_pid: process.parent().map(|p| p.as_u32()),
                start_time: process.start_time(),
                status: process.status().to_string(),
                user: process.user_id()
                    .and_then(|id| self.system.get_user_by_id(id)
                    .map(|user| user.name().to_string())),
                nice: nice_value as i32, // Correct casting for nice value
                startTime: formatted_time,
            });
        }
        
        // Sort by CPU usage (descending)
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        
        processes
    }
    
}
// Function to format the timestamp
fn format_timestamp(timestamp: u64) -> String {
    // The timestamp from sysinfo is usually in seconds since boot
    // We need to convert it to a DateTime object
    match Local.timestamp_opt(timestamp as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%H:%M:%S").to_string(),
        _ => "00:00:00".to_string() // Fallback if conversion fails
    }
}
