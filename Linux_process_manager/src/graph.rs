use crate::process::ProcessManager;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// Import Ratatui components
use ratatui::{
    Terminal,
    backend::Backend,
    widgets::{Block, Borders, Dataset, GraphType, Chart, Paragraph},
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    text::{Span, Line},
};

// Import Ratatui's color separately to avoid confusion
use ratatui::style::{Style, Modifier, Color as RatatuiColor};

// Import Crossterm components with explicit namespace
use crossterm::{
    style::{Color as CrosstermColor, SetForegroundColor, SetBackgroundColor, ResetColor, Attribute, SetAttribute},
    ExecutableCommand, 
    QueueableCommand,
};

// Graph data structure to store historical data points
pub struct GraphData {
    // CPU usage history (total system)
    cpu_history: VecDeque<f32>,
    // Memory usage history (in MB)
    memory_history: VecDeque<u64>,
    // Number of data points to keep
    max_points: usize,
    // Last update time
    last_update: Instant,
    // Update interval in milliseconds
    update_interval: Duration,
}

impl GraphData {
    pub fn new(max_points: usize, update_interval_ms: u64) -> Self {
        GraphData {
            cpu_history: VecDeque::with_capacity(max_points),
            memory_history: VecDeque::with_capacity(max_points),
            max_points,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(update_interval_ms),
        }
    }

    // Update graph data with current system metrics
    pub fn update(&mut self, process_manager: &ProcessManager) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_interval {
            return;
        }

        // Get current processes
        let processes = process_manager.get_processes();
        
        // Calculate total CPU usage (sum of all processes)
        let total_cpu: f32 = processes.iter().map(|p| p.cpu_usage).sum();
        
        // Calculate total memory usage in MB
        let total_memory: u64 = processes.iter().map(|p| p.memory_usage).sum::<u64>() / (1024 * 1024);
        
        // Add new data points with x-coordinate (time) and y-coordinate (value)
        self.cpu_history.push_back(total_cpu);
        self.memory_history.push_back(total_memory);
        
        // Remove oldest data points if we exceed max capacity
        if self.cpu_history.len() > self.max_points {
            self.cpu_history.pop_front();
        }
        
        if self.memory_history.len() > self.max_points {
            self.memory_history.pop_front();
        }
        
        self.last_update = now;
    }

    // Get CPU history data as slice of (x,y) points for Ratatui Chart
    pub fn get_cpu_history(&self) -> &VecDeque<f32> {
        &self.cpu_history
    }

    // Get memory history data as slice of (x,y) points for Ratatui Chart
    pub fn get_memory_history(&self) -> &VecDeque<u64> {
        &self.memory_history
    }
}

pub fn render_graph_dashboard(
    frame: &mut ratatui::Frame,
    process_manager: &ProcessManager,
    graph_data: &GraphData,
    stats_scroll_offset: usize,  // Add scroll offset parameter
) {
    let size = frame.size();
    
    // Calculate minimum sizes and adjust layout based on available space
    let min_graph_height = 10;
    let min_stats_height = 6;
    let total_min_height = min_graph_height * 2 + min_stats_height;

    let graph_height = if size.height >= total_min_height {
        // If we have enough space, use percentages
        Constraint::Percentage(40)
    } else {
        // Otherwise use minimum size
        Constraint::Length(min_graph_height)
    };

    // Create vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            graph_height,  // CPU Graph
            graph_height,  // Memory Graph
            Constraint::Min(min_stats_height),  // Stats
        ])
        .split(size);

    render_cpu_graph(frame, chunks[0], graph_data);
    render_memory_graph(frame, chunks[1], graph_data);
    render_stats(frame, chunks[2], process_manager, stats_scroll_offset);
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

    // Determine number of y-axis labels based on height
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
fn render_stats(
    frame: &mut ratatui::Frame,
    area: Rect,
    process_manager: &ProcessManager,
    stats_scroll_offset: usize,  // Add scroll offset parameter
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
            format!("Total processes: {}", process_count),
            Style::default().fg(RatatuiColor::Green)
        )]),
        Line::from(vec![Span::styled(
            format!("Total CPU: {:.2}%", total_cpu),
            Style::default().fg(if total_cpu > 75.0 {
                RatatuiColor::Red
            } else if total_cpu > 50.0 {
                RatatuiColor::Yellow
            } else {
                RatatuiColor::Green
            })
        )]),
        Line::from(vec![Span::styled(
            format!("Total Memory: {}MB", total_memory),
            Style::default().fg(RatatuiColor::Blue)
        )]),
        Line::from(vec![Span::styled(
            "TOP PROCESSES BY CPU",
            Style::default().fg(RatatuiColor::White).add_modifier(Modifier::BOLD)
        )]),
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
