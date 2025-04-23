use sysinfo::{ProcessExt, System, SystemExt, PidExt, UserExt};
use procfs::process::Process as ProcfsProcess; // Import procfs for nice value
use std::convert::TryInto; // Import the try_into function
use chrono::{DateTime, Local, TimeZone};
use libc::{self, c_int};

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
    processes: Vec<ProcessInfo>,
    sort_mode: Option<String>,
    sort_ascending: bool,
    filter_mode: Option<String>,
    filter_value: Option<String>,
}

impl ProcessManager {
    pub fn new() -> Self {
        let mut system = System::new_all(); 
        system.refresh_all(); 
        ProcessManager { 
            system,
            processes: Vec::new(),
            sort_mode: None,
            sort_ascending: true,
            filter_mode: None,
            filter_value: None,
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.update_processes();
        // Re-apply sort if there is an active sort mode
        if let Some(mode) = self.sort_mode.clone() {
            self.sort_processes(&mode);
        }
    }

    pub fn set_filter(&mut self, mode: Option<String>, value: Option<String>) {
        self.filter_mode = mode;
        self.filter_value = value;
        self.update_processes(); // Refresh to apply filter
    }

    fn update_processes(&mut self) {
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
            let proc_info = ProcessInfo {
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
                nice: nice_value as i32,
                startTime: formatted_time,
            };

            // Apply filter if set
            if let (Some(mode), Some(value)) = (&self.filter_mode, &self.filter_value) {
                let should_include = match mode.as_str() {
                    "user" => proc_info.user.as_ref().map_or(false, |u| u.contains(value)),
                    "name" => proc_info.name.to_lowercase().contains(&value.to_lowercase()),
                    "pid" => proc_info.pid.to_string().contains(value),
                    "ppid" => proc_info.parent_pid.map_or(false, |p| p.to_string().contains(value)),
                    _ => true,
                };
                if !should_include {
                    continue;
                }
            }

            processes.push(proc_info);
        }
        
        self.processes = processes;

        // Re-apply sort if there is an active sort mode
        if let Some(mode) = self.sort_mode.clone() {
            self.sort_processes(&mode);
        }
    }

    pub fn get_processes(&self) -> &Vec<ProcessInfo> {
        &self.processes
    }

    pub fn get_processes_mut(&mut self) -> &mut Vec<ProcessInfo> {
        &mut self.processes
    }

    pub fn set_sort(&mut self, mode: &str, ascending: bool) {
        self.sort_mode = Some(mode.to_string());
        self.sort_ascending = ascending;
        self.sort_processes(mode);
    }

    fn sort_processes(&mut self, mode: &str) {
        match mode {
            "pid" => {
                if self.sort_ascending {
                    self.processes.sort_by_key(|p| p.pid);
                } else {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.pid));
                }
            }
            "mem" => {
                if self.sort_ascending {
                    self.processes.sort_by_key(|p| p.memory_usage);
                } else {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.memory_usage));
                }
            }
            "ppid" => {
                if self.sort_ascending {
                    self.processes.sort_by_key(|p| p.parent_pid.unwrap_or(0));
                } else {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.parent_pid.unwrap_or(0)));
                }
            }
            "start" => {
                if self.sort_ascending {
                    self.processes.sort_by(|a, b| a.startTime.cmp(&b.startTime));
                } else {
                    self.processes.sort_by(|a, b| b.startTime.cmp(&a.startTime));
                }
            }
            "nice" => {
                if self.sort_ascending {
                    self.processes.sort_by_key(|p| p.nice);
                } else {
                    self.processes.sort_by_key(|p| std::cmp::Reverse(p.nice));
                }
            }
            "cpu" => {
                if self.sort_ascending {
                    self.processes.sort_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
                } else {
                    self.processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
                }
            }
            _ => {}
        }
    }

    pub fn set_niceness(&self, pid: u32, nice: i32) -> std::io::Result<()> {
        // Validate niceness range
        if nice < -20 || nice > 19 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Nice value must be between -20 and 19"
            ));
        }

        // Check privileges if setting negative nice
        if nice < 0 && unsafe { libc::geteuid() } != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Root privileges required for negative nice values (use sudo)"
            ));
        }
        let temp_pid: libc::id_t = pid;

        // SAFETY: This is safe because we're passing valid arguments
        let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, temp_pid, nice as c_int) };
        
        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn stop_process(&self, pid: u32) -> std::io::Result<()> {
        use libc::{kill, pid_t, SIGSTOP};
        
        let temp_pid: pid_t = pid as pid_t;
        
        // SAFETY: This is safe because we're passing valid arguments
        let result = unsafe { kill(temp_pid, SIGSTOP) };
        
        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }
        
        Ok(())
    }
    

    pub fn kill_process(&self, pid: u32) -> std::io::Result<()> {
        use libc::{kill, pid_t, SIGKILL};
        
        let temp_pid: pid_t = pid as pid_t;
        
        // SAFETY: This is safe because we're passing valid arguments
        let result = unsafe { kill(temp_pid, SIGKILL) };
        
        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }
        
        Ok(())
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
