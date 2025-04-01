use sysinfo::{ProcessExt, System, SystemExt, PidExt};

#[derive(Clone)] // ProcessInfo struct to hold process information
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
    pub status: String,
    pub user: Option<String>,
}

/// ProcessManager struct to manage system processes
/// It uses the sysinfo crate to gather information about the system processes.
/// The struct contains a System object that is used to refresh and retrieve process information.
/// The `refresh` method updates the process information.
/// The `get_processes` method retrieves a list of processes with their details.
/// The `print_processes` method prints the process information in a formatted manner.
/// The `ProcessInfo` struct contains fields for process ID, name, CPU usage, memory usage,
/// parent process ID, start time, status, and user.
/// The `ProcessManager` struct is initialized with a new `System` object.
/// The `get_processes` method returns a vector of `ProcessInfo` structs.
/// The `print_processes` method prints the process information in a formatted manner.
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
                user: process.user_id().map(|id| id.to_string()),
            });
        }
        
        // Sort by CPU usage (descending)
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
        
        processes
    }
    // Very Basic print function for processes testing
    pub fn print_processes(&self) {
        let processes = self.get_processes();
        
        println!("{:<6} {:<15} {:>5} {:>10} {:>10} {:>12} {:>10}", 
                 "PID", "NAME", "CPU%", "MEM(KB)", "PPID", "START", "USER");

        for process in processes.iter().take(20) {
            println!("{:<6} {:<15} {:>5.1} {:>10} {:>10} {:>12} {:>10}",
                     process.pid,
                     process.name,
                     process.cpu_usage,
                     process.memory_usage / 1024,
                     process.parent_pid.unwrap_or(0),
                     process.start_time,
                     process.user.clone().unwrap_or("N/A".to_string()));
        }
    }
}

