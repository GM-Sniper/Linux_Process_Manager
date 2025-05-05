use crate::process::ProcessManager;
use crate::process::ProcessInfo;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
// use std::collections::HashMap; //delete after debugging

// Import Ratatui components
use ratatui::{
    backend::Backend,
    widgets::{Block, Borders, Dataset, GraphType, Chart, Paragraph},
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    text::{Span, Line},
};

// Import Ratatui's color separately to avoid confusion
use ratatui::style::{Style, Modifier, Color as RatatuiColor};

// Import Crossterm components with explicit namespace
// use crossterm::{
//     style::{Color as CrosstermColor, SetForegroundColor, SetBackgroundColor, ResetColor, Attribute, SetAttribute},
//     ExecutableCommand, 
//     QueueableCommand,
// };

use crate::ui::StatisticsTab;  // Add this at the top with other imports

// Add this struct at the top with other structs
pub struct CpuInfo {
    pub usage: f32,
    last_idle: u64,
    last_total: u64,
}

impl CpuInfo {
    fn new() -> Self {
        Self {
            usage: 0.0,
            last_idle: 0,
            last_total: 0,
        }
    }
}

// Modify GraphData struct
pub struct GraphData {
    cpu_history: VecDeque<f32>,
    memory_history: VecDeque<u64>,
    max_points: usize,
    last_update: Instant,
    update_interval: Duration,
    cpu_infos: Vec<CpuInfo>,  // Keep this for per-core display
}

impl GraphData {
    pub fn new(max_points: usize, update_interval_ms: u64) -> Self {
        GraphData {
            cpu_history: VecDeque::with_capacity(max_points),
            memory_history: VecDeque::with_capacity(max_points),
            max_points,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(update_interval_ms),
            cpu_infos: (0..get_cpu_count()).map(|_| CpuInfo::new()).collect(),
        }
    }

    fn update_cpu_info(&mut self) {
        if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
            let lines: Vec<&str> = stat.lines().collect();
            
            // Handle individual cores for the CPU bars display
            for (i, cpu_info) in self.cpu_infos.iter_mut().enumerate() {
                if let Some(line) = lines.get(i + 1) {  // Skip first line (aggregate CPU)
                    if line.starts_with("cpu") {
                        let values: Vec<u64> = line.split_whitespace()
                            .skip(1)  // Skip "cpu" prefix
                            .filter_map(|val| val.parse().ok())
                            .collect();

                        if values.len() >= 4 {
                            let idle = values[3];
                            let total: u64 = values.iter().sum();

                            let idle_delta = idle - cpu_info.last_idle;
                            let total_delta = total - cpu_info.last_total;

                            if total_delta > 0 {
                                cpu_info.usage = 100.0 * (1.0 - (idle_delta as f32 / total_delta as f32));
                            }

                            cpu_info.last_idle = idle;
                            cpu_info.last_total = total;
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, process_manager: &ProcessManager) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }

        // Update CPU info for the per-core display
        self.update_cpu_info();
        
        // Get total CPU usage from all processes
        let total_cpu: f32 = process_manager.get_processes()
            .iter()
            .map(|p| p.cpu_usage)
            .sum();
        
        // Add to history
        self.cpu_history.push_back(total_cpu);
        
        // Calculate total memory usage in MB
        let total_memory: u64 = process_manager.get_processes()
            .iter()
            .map(|p| p.memory_usage)
            .sum::<u64>() / (1024 * 1024);
        
        self.memory_history.push_back(total_memory);
        
        if self.cpu_history.len() > self.max_points {
            self.cpu_history.pop_front();
        }
        
        if self.memory_history.len() > self.max_points {
            self.memory_history.pop_front();
        }
        
        self.last_update = now;
    }

    pub fn get_cpu_infos(&self) -> &[CpuInfo] {
        &self.cpu_infos
    }

    pub fn get_cpu_history(&self) -> &VecDeque<f32> {
        &self.cpu_history
    }

    pub fn get_memory_history(&self) -> &VecDeque<u64> {
        &self.memory_history
    }
}

pub fn render_graph_dashboard(
    frame: &mut ratatui::Frame,
    process_manager: &ProcessManager,
    graph_data: &GraphData,
    stats_scroll_offset: usize,
    current_tab: &StatisticsTab,  // Add current_tab parameter
) {
    let size = frame.size();
    
    // Create main layout with tabs and content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs and navigation help
            Constraint::Min(size.height.saturating_sub(3)),  // Content
        ])
        .split(size);
    
    // Render tabs
    render_tabs(frame, main_chunks[0], current_tab);

    // Render content based on current tab
    match current_tab {
        StatisticsTab::Graphs => render_graphs_tab(frame, main_chunks[1], process_manager, graph_data),
        StatisticsTab::SystemStats => render_system_stats_tab(frame, main_chunks[1], process_manager, stats_scroll_offset),
        StatisticsTab::Information => render_information_tab(frame, main_chunks[1], process_manager),
    }
}

fn render_tabs(frame: &mut ratatui::Frame, area: Rect, current_tab: &StatisticsTab) {
    // Get the current tab name
    let current_tab_name = match current_tab {
        StatisticsTab::Graphs => "CPU & Memory Graphs",
        StatisticsTab::SystemStats => "System Statistics",
        StatisticsTab::Information => "System Information",
    };

    let title = Line::from(vec![
        Span::styled("Current View: ", Style::default().fg(RatatuiColor::White)),
        Span::styled(current_tab_name, 
            Style::default()
                .fg(RatatuiColor::Cyan)  // Changed to Cyan for better visibility
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),  // Added underline for emphasis
        Span::raw(" "),
        Span::styled("[1-3] Switch Views ", Style::default().fg(RatatuiColor::Yellow)),
        Span::styled("[S/Esc] Return", Style::default().fg(RatatuiColor::Blue))
    ]);

    let header = Paragraph::new(title)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_graphs_tab(
    frame: &mut ratatui::Frame,
    area: Rect,
    process_manager: &ProcessManager,
    graph_data: &GraphData,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // CPU Bars (2 rows of 8 CPUs)
            Constraint::Length(3),   // Memory/Swap Bars with spacing
            Constraint::Percentage(45),  // CPU Graph
            Constraint::Percentage(45),  // Memory Graph
        ])
        .split(area);

    // Render CPU bars (similar to htop)
    render_cpu_bars(frame, chunks[0], process_manager, graph_data);
    
    // Create a sub-layout for memory bars with spacing
    let mem_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // Memory bar
            Constraint::Length(1),   // Spacing
            Constraint::Length(1),   // Swap bar
        ])
        .split(chunks[1]);

    // Render Memory/Swap bars with spacing
    render_memory_bars(frame, mem_chunks[0], mem_chunks[2], process_manager);

    // Render the graphs
    render_cpu_graph(frame, chunks[2], graph_data);
    render_memory_graph(frame, chunks[3], graph_data);
}

fn render_cpu_bars(frame: &mut ratatui::Frame, area: Rect, process_manager: &ProcessManager, graph_data: &GraphData) {
    let num_cpus = get_cpu_count();
    let cpus_per_row = 8;
    let num_rows = (num_cpus + cpus_per_row - 1) / cpus_per_row;
    
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(2); num_rows])
        .split(area);

    for row in 0..num_rows {
        let start_cpu = row * cpus_per_row;
        let end_cpu = ((row + 1) * cpus_per_row).min(num_cpus);
        let num_cpus_this_row = end_cpu - start_cpu;

        let cpu_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage((100 / cpus_per_row) as u16); cpus_per_row])
            .split(row_chunks[row]);

        for (i, chunk) in cpu_chunks.iter().take(num_cpus_this_row).enumerate() {
            let cpu_index = start_cpu + i;
            let cpu_usage = graph_data.get_cpu_infos().get(cpu_index).map_or(0.0, |info| info.usage);

            // Create a vertical bar using Unicode box-drawing characters
            let bar_height = ((cpu_usage / 100.0) * 8.0).round() as usize;
            let bar = "█".repeat(bar_height);
            let empty = "░".repeat(8 - bar_height);
            let vertical_bar = format!("{}{}", bar, empty);

            let label = format!("{:>2} [{:>3}%]", cpu_index, cpu_usage as u16);
            let text = vec![
                Line::from(vec![
                    Span::styled(label, Style::default().fg(RatatuiColor::White)),
                ]),
                Line::from(vec![
                    Span::styled(vertical_bar, Style::default().fg(get_usage_color(cpu_usage)))
                ])
            ];

            let cpu_widget = Paragraph::new(text)
                .alignment(Alignment::Left);

            frame.render_widget(cpu_widget, *chunk);
        }
    }
}

fn render_memory_bars(
    frame: &mut ratatui::Frame,
    mem_area: Rect,
    swap_area: Rect,
    process_manager: &ProcessManager
) {
    // Calculate memory usage
    let total_memory = process_manager.get_processes()
        .iter()
        .map(|p| p.memory_usage)
        .sum::<u64>() / (1024 * 1024);  // Convert to MB

    // Read total system memory from /proc/meminfo
    let total_system_memory = if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        meminfo.lines()
            .find(|line| line.starts_with("MemTotal:"))
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(0) / 1024  // Convert KB to MB
    } else {
        0
    };

    let memory_percentage = if total_system_memory > 0 {
        ((total_memory as f64 / total_system_memory as f64) * 100.0) as u16
    } else {
        0
    };

    // Memory bar with compact format
    let memory_gauge = ratatui::widgets::Gauge::default()
        .gauge_style(Style::default().fg(get_usage_color(memory_percentage as f32)))
        .percent(memory_percentage)
        .label(format!("Mem [{:>4}M/{:>4}M]", total_memory, total_system_memory));

    // Swap bar (reading from /proc/swaps)

    let (swap_used, swap_total) = get_swap_info();
    // let swap_percentage = if swap_total > 0 {
    //     ((swap_used as f64 / swap_total as f64) * 100.0) as u16
    // } else {
    //     0
    // };
    
    //fixes the panic that happens when screen is not full
    let swap_percentage = if swap_total > 0 && swap_used <= swap_total {
        let ratio = swap_used as f64 / swap_total as f64;
        let percent = (ratio * 100.0).clamp(0.0, 100.0).round();
        percent as u16
    } else {
        0
    };
    
    

    let swap_gauge = ratatui::widgets::Gauge::default()
        .gauge_style(Style::default().fg(get_usage_color(swap_percentage as f32)))
        .percent(swap_percentage)
        .label(format!("Swp [{:>4}M/{:>4}M]", swap_used, swap_total));

    frame.render_widget(memory_gauge, mem_area);
    frame.render_widget(swap_gauge, swap_area);
}

fn get_usage_color(usage: f32) -> RatatuiColor {
    match usage as u16 {
        0..=50 => RatatuiColor::Green,
        51..=75 => RatatuiColor::Yellow,
        76..=90 => RatatuiColor::Red,
        _ => RatatuiColor::LightRed,
    }
}

fn get_swap_info() -> (u64, u64) {
    if let Ok(swaps) = std::fs::read_to_string("/proc/swaps") {
        if let Some(swap_line) = swaps.lines().nth(1) {
            let parts: Vec<&str> = swap_line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(total), Ok(used)) = (
                    parts[2].parse::<u64>(),
                    parts[3].parse::<u64>(),
                ) {
                    return (used / 1024, total / 1024);  // Convert KB to MB
                }
            }
        }
    }
    (0, 0)
}

fn render_system_stats_tab(
    frame: &mut ratatui::Frame,
    area: Rect,
    process_manager: &ProcessManager,
    stats_scroll_offset: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Load Average explanation
            Constraint::Length(12),  // Detailed System Stats
            Constraint::Min(10),     // Process Stats
        ])
        .split(area);

    // Load Average explanation
    let load_avg = get_load_average();
    let load_text = vec![
        Line::from(vec![
            Span::styled("Load Average: ", Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:.2}, {:.2}, {:.2}", load_avg.0, load_avg.1, load_avg.2), 
                Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(" (1, 5, 15 min averages)")
        ]),
        Line::from(vec![
            Span::raw("Number of jobs in the run queue or waiting for disk I/O, relative to number of cores")
        ])
    ];

    let load_widget = Paragraph::new(load_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(load_widget, chunks[0]);

    // Detailed System Statistics
    let processes = process_manager.get_processes();
    let state_counts = get_process_state_counts(&processes);
    let (mem_total, mem_used, mem_free, mem_cached) = get_memory_info();
    let (disk_total, disk_used) = get_disk_stats();
    
    let total_cpu: f32 = processes.iter().map(|p| p.cpu_usage).sum();
    let total_memory: u64 = processes.iter().map(|p| p.memory_usage).sum::<u64>() / (1024 * 1024);
    
    let detailed_stats = vec![
        Line::from(vec![
            Span::styled("DETAILED SYSTEM STATISTICS", 
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("Process States: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("Running: {}", state_counts.get("Running").unwrap_or(&0)), 
                Style::default().fg(RatatuiColor::Green)),
            Span::raw(" | "),
            Span::styled(format!("Sleeping: {}", state_counts.get("Sleeping").unwrap_or(&0)), 
                Style::default().fg(RatatuiColor::Blue)),
            Span::raw(" | "),
            Span::styled(format!("Stopped: {}", state_counts.get("Stopped").unwrap_or(&0)), 
                Style::default().fg(RatatuiColor::Yellow)),
            Span::raw(" | "),
            Span::styled(format!("Zombie: {}", state_counts.get("Zombie").unwrap_or(&0)), 
                Style::default().fg(RatatuiColor::Red))
        ]),
        Line::from(vec![
            Span::styled("Memory Details: ", Style::default().fg(RatatuiColor::Gray)),
            Span::raw(format!("Total: {}MB | ", mem_total / 1024)),
            Span::styled(format!("Used: {}MB", mem_used / 1024), 
                get_usage_style((mem_used as f64 / mem_total as f64) * 100.0)),
            Span::raw(format!(" | Free: {}MB | Cached: {}MB", mem_free / 1024, mem_cached / 1024))
        ]),
        Line::from(vec![
            Span::styled("Disk Usage: ", Style::default().fg(RatatuiColor::Gray)),
            Span::raw(format!("Total: {}MB | ", disk_total)),
            Span::styled(format!("Used: {}MB", disk_used), 
                get_usage_style((disk_used as f64 / disk_total as f64) * 100.0)),
            Span::raw(format!(" | Free: {}MB", disk_total.saturating_sub(disk_used)))
        ]),
        Line::from(vec![
            Span::styled("CPU Cores: ", Style::default().fg(RatatuiColor::Gray)),
            Span::raw(format!("{} | ", get_cpu_count())),
            Span::styled("Current Usage: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{:.1}%", total_cpu), 
                get_usage_style(total_cpu as f64))
        ]),
        Line::from(vec![
            Span::styled("System Uptime: ", Style::default().fg(RatatuiColor::Gray)),
            Span::raw(get_system_uptime())
        ]),
    ];

    let detailed_stats_widget = Paragraph::new(detailed_stats)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(detailed_stats_widget, chunks[1]);

    // Process statistics
    let mut stats_text = vec![
        Line::from(vec![Span::styled(
            "TOP TASKS BY CPU",
            Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)
        )]),
    ];

    // Sort processes by CPU usage
    let mut sorted_processes = processes.to_vec();
    sorted_processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

    // Add top 10 processes by CPU usage
    for (i, process) in sorted_processes.iter().take(10).enumerate() {
        stats_text.push(Line::from(vec![Span::styled(
            format!("{}. {} (PID: {}) - CPU: {:.2}% | MEM: {}MB | Status: {}",
                i + 1,
                process.name,
                process.pid,
                process.cpu_usage,
                process.memory_usage / (1024 * 1024),
                process.status.trim()
            ),
            Style::default().fg(RatatuiColor::Yellow)
        )]));
    }

    // Sort by memory usage for memory section
    sorted_processes.sort_by_key(|p| std::cmp::Reverse(p.memory_usage));

    stats_text.push(Line::from(vec![Span::styled(
        "\nTOP TASKS BY MEMORY",
        Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)
    )]));

    // Add top 10 processes by memory usage
    for (i, process) in sorted_processes.iter().take(10).enumerate() {
        stats_text.push(Line::from(vec![Span::styled(
            format!("{}. {} (PID: {}) - MEM: {}MB | CPU: {:.2}% | Status: {}",
                i + 1,
                process.name,
                process.pid,
                process.memory_usage / (1024 * 1024),
                process.cpu_usage,
                process.status.trim()
            ),
            Style::default().fg(RatatuiColor::Blue)
        )]));
    }

    // Calculate maximum scroll offset
    let max_scroll = stats_text.len().saturating_sub(area.height as usize);
    let scroll_offset = stats_scroll_offset.min(max_scroll);

    // Create the scrollable stats widget
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Process Details (↑↓ to scroll) [{}/{}]", scroll_offset + 1, max_scroll + 1)))
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(stats_widget, chunks[2]);
}

fn render_information_tab(
    frame: &mut ratatui::Frame,
    area: Rect,
    process_manager: &ProcessManager,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),  // System Information
            Constraint::Length(16),  // CPU Information
            Constraint::Length(12),  // Memory Information
            Constraint::Length(10),  // System Load
            Constraint::Min(5),      // Navigation Help
        ])
        .split(area);

    // System Information (existing code with boot time)
    let (boot_time, last_reboot) = get_boot_time();
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    // Create bindings for values that need to live longer
    let os_info = get_os_info();
    let kernel_version = std::fs::read_to_string("/proc/version").unwrap_or_default();
    let uptime = get_system_uptime();

    let sys_info = vec![
        Line::from(vec![
            Span::styled("System Information", 
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("Hostname: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&hostname, Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("OS: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&os_info, Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Kernel: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&kernel_version, Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Boot Time: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&boot_time, Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Last Reboot: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&last_reboot, Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Uptime: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&uptime, Style::default().fg(RatatuiColor::White))
        ]),
    ];

    let sys_info_widget = Paragraph::new(sys_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(sys_info_widget, chunks[0]);

    // Enhanced CPU Information
    let (cpu_model, _, cpu_cache) = get_cpu_details();
    let core_freqs = get_per_core_freq();
    let load_avg = get_load_average();
    let cpu_temp = get_cpu_temp();
    let (ctxt, processes, procs_running, procs_blocked, interrupts) = get_cpu_stats();

    let mut cpu_info = vec![
        Line::from(vec![
            Span::styled("CPU Information", 
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("Model: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(&cpu_model, Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Cores: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{} (Physical)", get_cpu_count()), 
                Style::default().fg(RatatuiColor::White))
        ]),
    ];

    // Add temperature if available
    if let Some(temp) = cpu_temp {
        cpu_info.push(Line::from(vec![
            Span::styled("Temperature: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{:.1}°C", temp), 
                Style::default().fg(if temp > 80.0 { 
                    RatatuiColor::Red 
                } else if temp > 60.0 { 
                    RatatuiColor::Yellow 
                } else { 
                    RatatuiColor::Green 
                }))
        ]));
    }

    // Add per-core frequencies
    for (i, freq) in core_freqs.iter().enumerate() {
        cpu_info.push(Line::from(vec![
            Span::styled(format!("Core {} Freq: ", i), Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{:.0} MHz", freq), Style::default().fg(RatatuiColor::White))
        ]));
    }

    // Add load and stats
    cpu_info.extend(vec![
        Line::from(vec![
            Span::styled("Load Average: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{:.2}, {:.2}, {:.2} (1, 5, 15 min)", 
                load_avg.0, load_avg.1, load_avg.2), 
                Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Context Switches: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{}/s", ctxt), Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Interrupts: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{}/s", interrupts), Style::default().fg(RatatuiColor::White))
        ]),
    ]);

    let cpu_info_widget = Paragraph::new(cpu_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(cpu_info_widget, chunks[1]);

    // Enhanced Memory Information
    let (mem_total, mem_used, mem_free, swap_total, swap_used) = get_memory_details();
    let (pgfault, pswpin, pswpout, iowait) = get_vm_stats();

    let mem_info = vec![
        Line::from(vec![
            Span::styled("Memory Information", 
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("RAM: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(
                format!("Total: {} | Used: {} | Free: {}", 
                    format_bytes(mem_total * 1024),
                    format_bytes(mem_used * 1024),
                    format_bytes(mem_free * 1024)
                ),
                Style::default().fg(RatatuiColor::White)
            )
        ]),
        Line::from(vec![
            Span::styled("Swap: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(
                format!("Total: {} | Used: {} | Free: {}", 
                    format_bytes(swap_total * 1024),
                    format_bytes(swap_used * 1024),
                    format_bytes((swap_total - swap_used) * 1024)
                ),
                Style::default().fg(RatatuiColor::White)
            )
        ]),
        Line::from(vec![
            Span::styled("Page Faults: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{}/s", pgfault), Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("Swap I/O: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("In: {}/s | Out: {}/s", pswpin, pswpout), 
                Style::default().fg(RatatuiColor::White))
        ]),
    ];

    let mem_info_widget = Paragraph::new(mem_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(mem_info_widget, chunks[2]);

    // System Load Information
    let load_info = vec![
        Line::from(vec![
            Span::styled("System Load", 
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("Processes: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("Running: {} | Blocked: {} | Total: {}", 
                procs_running, procs_blocked, processes), 
                Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("I/O Wait: ", Style::default().fg(RatatuiColor::Gray)),
            Span::styled(format!("{} ticks", iowait), Style::default().fg(RatatuiColor::White))
        ]),
    ];

    let load_info_widget = Paragraph::new(load_info)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(load_info_widget, chunks[3]);

    // Navigation Help
    let help = vec![
        Line::from(vec![
            Span::styled("Navigation Help", 
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(RatatuiColor::Yellow)),
            Span::styled("Use [1-3] to switch between tabs", Style::default().fg(RatatuiColor::White))
        ]),
        Line::from(vec![
            Span::styled("• ", Style::default().fg(RatatuiColor::Yellow)),
            Span::styled("Press [S] or [Esc] to return to process list", Style::default().fg(RatatuiColor::White))
        ]),
    ];

    let help_widget = Paragraph::new(help)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    frame.render_widget(help_widget, chunks[4]);
}

fn get_os_info() -> String {
    std::fs::read_to_string("/etc/os-release")
        .map(|content| {
            let mut name = String::new();
            let mut version = String::new();
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    name = line.split('=').nth(1)
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string();
                }
                if line.starts_with("VERSION=") {
                    version = line.split('=').nth(1)
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string();
                }
            }
            if !name.is_empty() { name } else { "Unknown".to_string() }
        })
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_cpu_details() -> (String, String, String) { // Returns (model, frequency, cache)
    let mut model = String::new();
    let mut freq = String::new();
    let mut cache = String::new();

    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                model = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("cpu MHz") {
                freq = format!("{:.2} MHz", line.split(':').nth(1).unwrap_or("0").trim().parse::<f64>().unwrap_or(0.0));
            } else if line.starts_with("cache size") {
                cache = line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }
    }
    (model, freq, cache)
}

fn get_memory_details() -> (u64, u64, u64, u64, u64) { // Returns (total, used, free, swap_total, swap_used) in KB
    let mut mem_total = 0;
    let mut mem_free = 0;
    let mut mem_available = 0;
    let mut swap_total = 0;
    let mut swap_free = 0;

    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            match line.split_whitespace().next() {
                Some("MemTotal:") => mem_total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0),
                Some("MemFree:") => mem_free = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0),
                Some("MemAvailable:") => mem_available = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0),
                Some("SwapTotal:") => swap_total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0),
                Some("SwapFree:") => swap_free = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0),
                _ => {}
            }
        }
    }
    (mem_total, mem_total - mem_available, mem_free, swap_total, swap_total - swap_free)
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn get_system_uptime() -> String {
    if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs_str) = uptime.split_whitespace().next() {
            if let Ok(secs) = secs_str.parse::<f64>() {
                let days = (secs / 86400.0) as u64;
                let hours = ((secs % 86400.0) / 3600.0) as u64;
                let minutes = ((secs % 3600.0) / 60.0) as u64;
                return format!("{}d {}h {}m", days, hours, minutes);
            }
        }
    }
    "Unknown".to_string()
}

fn get_load_average() -> (f64, f64, f64) {
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        let values: Vec<f64> = loadavg
            .split_whitespace()
            .take(3)
            .filter_map(|s| s.parse().ok())
            .collect();
        if values.len() == 3 {
            return (values[0], values[1], values[2]);
        }
    }
    (0.0, 0.0, 0.0)
}

fn get_cpu_count() -> usize {
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        return cpuinfo.lines()
            .filter(|line| line.starts_with("processor"))
            .count();
    }
    1
}

fn get_usage_style(usage: f64) -> Style {
    match usage {
        u if u > 90.0 => Style::default().fg(RatatuiColor::Red),
        u if u > 70.0 => Style::default().fg(RatatuiColor::Yellow),
        _ => Style::default().fg(RatatuiColor::Green),
    }
}

fn render_cpu_graph(
    frame: &mut ratatui::Frame,
    area: Rect,
    graph_data: &GraphData,
) {
    let cpu_data: Vec<(f64, f64)> = graph_data
        .get_cpu_history()
        .iter()
        .enumerate()
        .map(|(i, &value)| (i as f64, value as f64))
        .collect();

    // Determine y-axis labels based on height
    let y_labels = if area.height > 15 {
        vec!["0%", "25%", "50%", "75%", "100%"]
    } else if area.height > 10 {
        vec!["0%", "50%", "100%"]
    } else {
        vec!["0%", "100%"]
    };

    let dataset = Dataset::default()
        .name("CPU Usage")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(RatatuiColor::Cyan))
        .data(&cpu_data);

    let chart = Chart::new(vec![dataset])
        .block(Block::default()
            .title("CPU Usage Over Time (%)")
            .borders(Borders::ALL))
        .x_axis(ratatui::widgets::Axis::default()
            .bounds([0.0, graph_data.max_points as f64])
            .labels(vec![]))
        .y_axis(ratatui::widgets::Axis::default()
            .bounds([0.0, 100.0])
            .labels(y_labels
                .into_iter()
                .map(Span::from)
                .collect()));

    frame.render_widget(chart, area);
}

fn render_memory_graph(
    frame: &mut ratatui::Frame,
    area: Rect,
    graph_data: &GraphData,
) {
    let memory_data: Vec<(f64, f64)> = graph_data
        .get_memory_history()
        .iter()
        .enumerate()
        .map(|(i, &value)| (i as f64, value as f64))
        .collect();

    let max_memory = memory_data
        .iter()
        .map(|&(_, y)| y)
        .fold(100.0_f64, |a, b| a.max(b));

    // Determine number of y-axis labels based on height
    let y_labels = if area.height > 15 {
        vec![
            format!("0"),
            format!("{:.0}", max_memory / 4.0),
            format!("{:.0}", max_memory / 2.0),
            format!("{:.0}", max_memory * 3.0 / 4.0),
            format!("{:.0}", max_memory),
        ]
    } else if area.height > 10 {
        vec![
            format!("0"),
            format!("{:.0}", max_memory / 2.0),
            format!("{:.0}", max_memory),
        ]
    } else {
        vec![
            format!("0"),
            format!("{:.0}", max_memory),
        ]
    };

    let dataset = Dataset::default()
        .name("Memory Usage")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(RatatuiColor::Green))
        .data(&memory_data);

    let chart = Chart::new(vec![dataset])
        .block(Block::default()
            .title("Memory Usage Over Time (MB)")
            .borders(Borders::ALL))
        .x_axis(ratatui::widgets::Axis::default()
            .bounds([0.0, graph_data.max_points as f64])
            .labels(vec![]))
        .y_axis(ratatui::widgets::Axis::default()
            .bounds([0.0, max_memory])
            .labels(y_labels
                .into_iter()
                .map(Span::from)
                .collect()));

    frame.render_widget(chart, area);
}

// Render system statistics
fn render_process_stats(
    frame: &mut ratatui::Frame,
    area: Rect,
    process_manager: &ProcessManager,
    stats_scroll_offset: usize,
) {
    let processes = process_manager.get_processes();
    
    // Get total CPU and memory
    let total_cpu: f32 = processes.iter().map(|p| p.cpu_usage).sum();
    let total_memory: u64 = processes.iter().map(|p| p.memory_usage).sum::<u64>() / (1024 * 1024);
    let process_count = processes.len();
    
    // Sort processes by CPU usage for top processes list
    let mut sorted_processes = processes.to_vec();
    sorted_processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));

    // Create the full statistics text
    let mut stats_text = vec![
        Line::from(vec![Span::styled(
            "SYSTEM STATISTICS",
            Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)
        )]),
        Line::from(vec![Span::styled(
            format!("Tasks: {}", process_count),  // Changed from "Total processes" to "Tasks"
            Style::default().fg(RatatuiColor::Green)
        )]),
        Line::from(vec![
            Span::styled(
                format!("Total CPU: {:.2}%", total_cpu),
                Style::default().fg(if total_cpu > 75.0 {
                    RatatuiColor::Red
                } else if total_cpu > 50.0 {
                    RatatuiColor::Yellow
                } else {
                    RatatuiColor::Green
                })
            )
        ]),
        Line::from(vec![
            Span::styled(
                format!("Total Memory: {}MB", total_memory),
                Style::default().fg(RatatuiColor::Blue)
            )
        ]),
        Line::from(vec![
            Span::styled(
                "TOP TASKS BY CPU",  // Changed from "TOP PROCESSES" to "TOP TASKS"
                Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)
            )
        ]),
    ];

    // Add top 10 processes by CPU usage
    for (i, process) in sorted_processes.iter().take(10).enumerate() {
        stats_text.push(Line::from(vec![Span::styled(
            format!("{}. {} (PID: {}) - CPU: {:.2}% | MEM: {}MB",
                i + 1,
                process.name,
                process.pid,
                process.cpu_usage,
                process.memory_usage / (1024 * 1024)
            ),
            Style::default().fg(RatatuiColor::Yellow)
        )]));
    }

    // Sort by memory usage for memory section
    sorted_processes.sort_by_key(|p| std::cmp::Reverse(p.memory_usage));

    stats_text.push(Line::from(vec![Span::styled(
        "TOP PROCESSES BY MEMORY",
        Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)
    )]));

    // Add top 10 processes by memory usage
    for (i, process) in sorted_processes.iter().take(10).enumerate() {
        stats_text.push(Line::from(vec![Span::styled(
            format!("{}. {} (PID: {}) - MEM: {}MB | CPU: {:.2}%",
                i + 1,
                process.name,
                process.pid,
                process.memory_usage / (1024 * 1024),
                process.cpu_usage
            ),
            Style::default().fg(RatatuiColor::Blue)
        )]));
    }

    // Calculate maximum scroll offset
    let max_scroll = stats_text.len().saturating_sub(area.height as usize);
    let scroll_offset = stats_scroll_offset.min(max_scroll);

    // Create the scrollable stats widget
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Statistics (↑↓ to scroll) [{}/{}]", scroll_offset + 1, max_scroll + 1)))
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(stats_widget, area);
}

// Add these helper functions at the top level
fn get_process_state_counts(processes: &[ProcessInfo]) -> std::collections::HashMap<String, usize> {
    let mut states = std::collections::HashMap::new();
    for process in processes {
        // Convert status to lowercase and trim for consistent matching
        let status = process.status.trim().to_lowercase();
        // Map the status to standard categories
        let mapped_status = match status.as_str() {
            "s" | "sleeping" => "Sleeping",
            "r" | "running" => "Running",
            "t" | "stopped" | "t (stopped)" => "Stopped",
            "z" | "zombie" => "Zombie",
            _ => "Other"
        };
        *states.entry(mapped_status.to_string()).or_insert(0) += 1;
    }
    
    // Ensure all states exist in the map
    for state in ["Running", "Sleeping", "Stopped", "Zombie"] {
        states.entry(state.to_string()).or_insert(0);
    }
    
    states
}

fn get_memory_info() -> (u64, u64, u64, u64) { // Returns (total, used, free, cached) in KB
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total = 0;
        let mut free = 0;
        let mut cached = 0;
        let mut buffers = 0;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("MemFree:") {
                free = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("Cached:") {
                cached = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            } else if line.starts_with("Buffers:") {
                buffers = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }
        let used = total - free - cached - buffers;
        return (total, used, free, cached + buffers);
    }
    (0, 0, 0, 0)
}

fn get_disk_stats() -> (u64, u64) { // Returns (total, used) in MB
    if let Ok(output) = std::process::Command::new("df")
        .arg("-BM")  // Force output in MB
        .arg("/")
        .output() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() > 1 {
                let stats: Vec<&str> = lines[1].split_whitespace().collect();
                if stats.len() >= 3 {
                    let total: u64 = stats[1].trim_end_matches('M').parse().unwrap_or(0);
                    let used: u64 = stats[2].trim_end_matches('M').parse().unwrap_or(0);
                    return (total, used);
                }
            }
        }
    }
    (0, 0)
}

// Add these new helper functions
fn get_cpu_temp() -> Option<f64> {
    if let Ok(temp) = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp") {
        if let Ok(temp_val) = temp.trim().parse::<u32>() {
            return Some(temp_val as f64 / 1000.0);
        }
    }
    None
}

fn get_per_core_freq() -> Vec<f64> {
    let mut freqs = Vec::new();
    let cpu_count = get_cpu_count();
    
    for i in 0..cpu_count {
        if let Ok(freq) = std::fs::read_to_string(format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_cur_freq", i)) {
            if let Ok(freq_val) = freq.trim().parse::<u32>() {
                freqs.push(freq_val as f64 / 1000.0); // Convert to MHz
            }
        }
    }
    freqs
}

fn get_cpu_stats() -> (u64, u64, u64, u64, u64) { // Returns (ctxt, processes, procs_running, procs_blocked, interrupts)
    let mut ctxt = 0;
    let mut processes = 0;
    let mut procs_running = 0;
    let mut procs_blocked = 0;
    let mut interrupts = 0;

    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        for line in stat.lines() {
            match line.split_whitespace().next() {
                Some("ctxt") => {
                    ctxt = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                Some("processes") => {
                    processes = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                Some("procs_running") => {
                    procs_running = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                Some("procs_blocked") => {
                    procs_blocked = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                Some("intr") => {
                    interrupts = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                _ => {}
            }
        }
    }
    (ctxt, processes, procs_running, procs_blocked, interrupts)
}

fn get_vm_stats() -> (u64, u64, u64, u64) { // Returns (page_faults, swap_in, swap_out, io_wait)
    let mut pgfault = 0;
    let mut pswpin = 0;
    let mut pswpout = 0;
    let mut iowait = 0;

    if let Ok(vmstat) = std::fs::read_to_string("/proc/vmstat") {
        for line in vmstat.lines() {
            match line.split_whitespace().next() {
                Some("pgfault") => {
                    pgfault = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                Some("pswpin") => {
                    pswpin = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                Some("pswpout") => {
                    pswpout = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                }
                _ => {}
            }
        }
    }

    // Get IO wait from /proc/stat
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        if let Some(cpu_line) = stat.lines().next() {
            let values: Vec<u64> = cpu_line.split_whitespace()
                .skip(1)
                .filter_map(|val| val.parse().ok())
                .collect();
            if values.len() >= 6 {
                iowait = values[4]; // iowait is the 5th value
            }
        }
    }

    (pgfault, pswpin, pswpout, iowait)
}

fn get_boot_time() -> (String, String) { // Returns (boot_time, last_reboot)
    let mut boot_time = String::from("Unknown");
    let mut last_reboot = String::from("Unknown");

    if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs_str) = uptime.split_whitespace().next() {
            if let Ok(secs) = secs_str.parse::<f64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as f64;
                let boot_timestamp = now - secs;
                
                // Format boot time
                let datetime = chrono::NaiveDateTime::from_timestamp_opt(boot_timestamp as i64, 0)
                    .unwrap_or_default();
                boot_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                
                // Try to get last reboot from wtmp (if available)
                if let Ok(output) = std::process::Command::new("last")
                    .arg("-x")
                    .arg("reboot")
                    .arg("-F")
                    .output() {
                    if let Ok(output_str) = String::from_utf8(output.stdout) {
                        if let Some(last_reboot_line) = output_str.lines().next() {
                            last_reboot = last_reboot_line.to_string();
                        }
                    }
                }
            }
        }
    }
    (boot_time, last_reboot)
}
