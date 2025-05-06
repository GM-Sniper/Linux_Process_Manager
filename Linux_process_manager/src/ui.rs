use crate::process;
use crate::graph;
use std::io::stdout;
use std::thread::sleep;
use std::time::Duration;
use process::ProcessManager;
use std::error::Error;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Table, Row, Cell,
        Dataset, GraphType, Chart,
    },
    layout::{Layout, Constraint, Direction, Alignment},
    style::{Style, Modifier, Color},
    text::{Line, Span},
    Frame,
};

// ViewMode enum to track current view
#[derive(PartialEq)]
enum ViewMode {
    ProcessList,
    Statistics,  // Renamed from GraphView
    FilterSort,
    Sort,
    Filter,
    FilterInput,
    KillStop,
    ChangeNice,
    PerProcessGraph, // Added for new feature
    ProcessLog,      // Added for new feature
    Help,            // Added for new feature
}

// Input state for various operations
struct InputState {
    pid_input: String,
    nice_input: String,
    filter_input: String,
    message: Option<(String, bool)>, // (message, is_error)
    message_timeout: Option<std::time::Instant>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pid_input: String::new(),
            nice_input: String::new(),
            filter_input: String::new(),
            message: None,
            message_timeout: None,
        }
    }
}

// NiceInputState enum to track the state of nice value input
#[derive(PartialEq)]
enum NiceInputState {
    SelectingPid,
    EnteringNice,
}

// KillStopInputState enum to track the state of kill/stop/continue input
#[derive(PartialEq)]
enum KillStopInputState {
    SelectingPid,
    EnteringAction,
}

// StatisticsTab enum to track the current statistics tab
#[derive(PartialEq)]
pub enum StatisticsTab {
    Graphs,
    Overview,
    CPU,
    Memory,
    PerProcessGraph, // New tab for per-process graphing
    ProcessLog,      // New tab for process logging
    Disk,
    Processes,
    Advanced,
    Help,            // New tab for help
}

// App state
struct App {
    process_manager: ProcessManager,
    graph_data: graph::GraphData,
    view_mode: ViewMode,
    scroll_offset: usize,
    display_limit: usize,
    input_state: InputState,
    sort_ascending: bool,
    sort_mode: Option<String>,
    filter_mode: Option<String>,
    stats_scroll_offset: usize,  // New field for statistics scrolling
    nice_input_state: NiceInputState,  // Track which input we're currently handling
    current_stats_tab: StatisticsTab,  // New field for tracking current statistics tab
    change_nice_scroll_offset: usize,
    selected_process_index: usize,
    per_process_graph_scroll_offset: usize,  // Add this
    selected_process_for_graph: Option<u32>,  // Add this
    kill_stop_input_state: KillStopInputState,  // Add this
}

impl App {
    fn new() -> Self {
        Self {
            process_manager: ProcessManager::new(),
            graph_data: graph::GraphData::new(60, 500),
            view_mode: ViewMode::ProcessList,
            scroll_offset: 0,
            display_limit: 20,
            input_state: InputState::default(),
            sort_ascending: true,
            sort_mode: None,
            filter_mode: None,
            stats_scroll_offset: 0,  // Initialize stats scroll offset
            nice_input_state: NiceInputState::SelectingPid,
            current_stats_tab: StatisticsTab::Graphs,  // Default to Graphs tab
            change_nice_scroll_offset: 0,
            selected_process_index: 0,
            per_process_graph_scroll_offset: 0,  // Add this
            selected_process_for_graph: None,    // Add this
            kill_stop_input_state: KillStopInputState::SelectingPid,  // Add this
        }
    }

    fn refresh(&mut self) {
        self.process_manager.refresh();
        self.graph_data.update(&self.process_manager);
    }
}


//ui_renderer
pub fn ui_renderer() -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        app.refresh();

        terminal.draw(|f| {
            match app.view_mode {
                ViewMode::ProcessList => draw_process_list(f, &app),
                ViewMode::Statistics => graph::render_graph_dashboard(
                    f,
                    &app.graph_data,
                    &app.current_stats_tab,
                    app.process_manager.get_processes(),
                ),
                ViewMode::FilterSort => draw_filter_sort_menu(f),
                ViewMode::Sort => draw_sort_menu(f, &app),
                ViewMode::Filter => draw_filter_menu(f),
                ViewMode::FilterInput => draw_filter_input_menu(f, &app),
                ViewMode::KillStop => draw_kill_stop_menu(f, &app),
                ViewMode::ChangeNice => draw_change_nice_menu(f, &app),
                ViewMode::PerProcessGraph => render_per_process_graph_tab(f, f.size(), &app),
                ViewMode::ProcessLog => {
                    let size = f.size();
                    let para = Paragraph::new("Process Log View (to be implemented)")
                        .block(Block::default().borders(Borders::ALL).title("Process Log"));
                    f.render_widget(para, size);
                },
                ViewMode::Help => {
                    let size = f.size();
                    let para = Paragraph::new("Help View (to be implemented)")
                        .block(Block::default().borders(Borders::ALL).title("Help"));
                    f.render_widget(para, size);
                },
            }
        })?;

        if handle_events(&mut app)? {
            break;
        }

        sleep(Duration::from_millis(100));
    }

    // Cleanup and restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    
    Ok(())
}

const PROCESS_TABLE_HEIGHT: usize = 12;

fn draw_process_list(f: &mut Frame, app: &App) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),     // Header
            Constraint::Min(size.height.saturating_sub(6)), // Process list
            Constraint::Length(3),   // Menu
        ])
        .split(size);

    // Get sort indicator for each column
    let get_sort_indicator = |column: &str| -> &str {
        if let Some(mode) = &app.sort_mode {
            if mode == column {
                if app.sort_ascending {
                    " ↑"
                } else {
                    " ↓"
                }
            } else {
                ""
            }
        } else {
            ""
        }
    };

    // Header
    let headers = [
        format!("PID{}", get_sort_indicator("pid")),
        format!("NAME{}", get_sort_indicator("name")),
        format!("CPU%{}", get_sort_indicator("cpu")),
        format!("MEM(MB){}", get_sort_indicator("mem")),
        format!("PPID{}", get_sort_indicator("ppid")),
        format!("START{}", get_sort_indicator("start")),
        format!("NICE{}", get_sort_indicator("nice")),
        format!("USER{}", get_sort_indicator("user")),
        "STATUS".to_string(),
    ];

    let header_cells = headers
        .iter()
        .map(|h| Cell::from(h.as_str()).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
    
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1);

    // Process rows
    let processes = app.process_manager.get_processes();
    let rows: Vec<Row> = processes
        .iter()
        .skip(app.scroll_offset)
        .take(app.display_limit)
        .enumerate()
        .map(|(i, process)| {
            let style = if i % 2 == 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Blue)
            };

            let memory_mb = process.memory_usage / (1024 * 1024);
            let cpu_style = match process.cpu_usage {
                c if c > 50.0 => Style::default().fg(Color::Red),
                c if c > 25.0 => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Green),
            };

            Row::new(vec![
                Cell::from(process.pid.to_string()).style(style),
                Cell::from(process.name.clone()).style(Style::default().fg(Color::Green)),
                Cell::from(format!("{:.2}%", process.cpu_usage)).style(cpu_style),
                Cell::from(format!("{}MB", memory_mb)).style(style),
                Cell::from(process.parent_pid.unwrap_or(0).to_string()).style(style),
                Cell::from(process.start_time_str.clone()).style(Style::default()),
                Cell::from(process.nice.to_string()).style(Style::default().fg(Color::Yellow)),
                Cell::from(process.user.clone().unwrap_or_default()).style(Style::default().fg(Color::Magenta)),
                Cell::from(process.status.trim()).style(get_status_style(&process.status)),
            ])
        })
        .collect();

    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .widths(&[
            Constraint::Length(8),  // PID
            Constraint::Length(20), // NAME
            Constraint::Length(8),  // CPU%
            Constraint::Length(10), // MEM
            Constraint::Length(8),  // PPID
            Constraint::Length(12), // START
            Constraint::Length(8),  // NICE
            Constraint::Length(12), // USER
            Constraint::Length(10), // STATUS
        ]);

    f.render_widget(table, chunks[1]);

    // Menu
    let menu_text = vec![
        Line::from(vec![
            Span::styled("[↑/↓] Scroll  ", Style::default().fg(Color::Cyan)),
            Span::raw("| "),
            Span::styled("[1] Filter/Sort  ", Style::default().fg(Color::Yellow)),
            Span::raw("| "),
            Span::styled("[2] Change Nice  ", Style::default().fg(Color::Green)),
            Span::raw("| "),
            Span::styled("[3] Kill/Stop  ", Style::default().fg(Color::Red)),
            Span::raw("| "),
            Span::styled("[4] Per-Process Graph  ", Style::default().fg(Color::Magenta)),
            Span::raw("| "),
            Span::styled("[5] Process Log  ", Style::default().fg(Color::Cyan)),
            Span::raw("| "),
            Span::styled("[6] Help  ", Style::default().fg(Color::Yellow)),
            Span::raw("| "),
            Span::styled("[S] Statistics  ", Style::default().fg(Color::Blue)),
            Span::raw("| "),
            Span::styled("[q] Quit", Style::default().fg(Color::White)),
        ]),
    ];

    let menu = Paragraph::new(menu_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);

    f.render_widget(menu, chunks[2]);
}

fn draw_filter_sort_menu(f: &mut Frame) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Menu items
            Constraint::Length(3),  // Status
        ])
        .split(size);

    // Title
    let title = Paragraph::new("Filter/Sort Menu")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu items
    let items = vec![
        ListItem::new(Span::styled("[1] Sort", Style::default().fg(Color::Yellow))),
        ListItem::new(Span::styled("[2] Filter", Style::default().fg(Color::Green))),
        ListItem::new(Span::styled("[←] Back", Style::default().fg(Color::Blue))),
    ];

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(menu, chunks[1]);
}

fn draw_sort_menu(f: &mut Frame, app: &App) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Menu items
            Constraint::Length(3),  // Status
        ])
        .split(size);

    // Title
    let title = Paragraph::new("Sort Menu")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu items
    let items = vec![
        ListItem::new(Span::styled("[1] Sort by PID", Style::default().fg(Color::Yellow))),
        ListItem::new(Span::styled("[2] Sort by Memory", Style::default().fg(Color::Green))),
        ListItem::new(Span::styled("[3] Sort by PPID", Style::default().fg(Color::Blue))),
        ListItem::new(Span::styled("[4] Sort by Start Time", Style::default().fg(Color::Magenta))),
        ListItem::new(Span::styled("[5] Sort by Nice Value", Style::default().fg(Color::Cyan))),
        ListItem::new(Span::styled("[6] Sort by CPU Usage", Style::default().fg(Color::Red))),
        ListItem::new(Span::styled("[a] Toggle Ascending/Descending", Style::default().fg(Color::White))),
        ListItem::new(Span::styled("[←] Back", Style::default().fg(Color::Blue))),
    ];

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(menu, chunks[1]);

    // Status
    let order_text = format!("Current Order: {}", if app.sort_ascending { "Ascending ↑" } else { "Descending ↓" });
    let status = Paragraph::new(order_text)
        .style(Style::default())
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status, chunks[2]);
}

fn draw_filter_menu(f: &mut Frame) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Menu items
        ])
        .split(size);

    // Title
    let title = Paragraph::new("Select Filter Type")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu items
    let items = vec![
        ListItem::new(Span::styled("[1] Filter by User", Style::default().fg(Color::Magenta))),
        ListItem::new(Span::styled("[2] Filter by Name", Style::default().fg(Color::Green))),
        ListItem::new(Span::styled("[3] Filter by PID", Style::default().fg(Color::Yellow))),
        ListItem::new(Span::styled("[4] Filter by PPID", Style::default().fg(Color::Cyan))),
        ListItem::new(Span::styled("[Esc] Clear Filter", Style::default().fg(Color::Red))),
        ListItem::new(Span::styled("[←] Back", Style::default().fg(Color::Blue))),
    ];

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(menu, chunks[1]);
}

fn draw_filter_input_menu(f: &mut Frame, app: &App) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Instructions
            Constraint::Length(3),  // Input
        ])
        .split(size);

    // Title
    let filter_type = match app.filter_mode.as_deref() {
        Some("user") => "User",
        Some("name") => "Process Name",
        Some("pid") => "PID",
        Some("ppid") => "Parent PID",
        _ => "Unknown",
    };
    let title = Paragraph::new(format!("Enter {} Filter", filter_type))
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Instructions
    let mut instructions = vec![
        ListItem::new(Span::styled(
            format!("Enter value to filter by {}", filter_type.to_lowercase()),
            Style::default().fg(Color::White)
        )),
        ListItem::new(Span::styled("[Enter] Apply Filter", Style::default().fg(Color::Green))),
        ListItem::new(Span::styled("[←] Back", Style::default().fg(Color::Blue))),
    ];

    if app.filter_mode.as_deref().map_or(false, |m| m == "pid" || m == "ppid") {
        instructions.insert(1, ListItem::new(Span::styled(
            "(Numbers only)",
            Style::default().fg(Color::Yellow)
        )));
    }

    let instructions_widget = List::new(instructions)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default());

    f.render_widget(instructions_widget, chunks[1]);

    // Input field
    let input_text = format!("Filter value: {}", app.input_state.filter_input);
    let input = Paragraph::new(input_text)
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(input, chunks[2]);
}

fn draw_kill_stop_menu(f: &mut Frame, app: &App) {
    let size = f.size();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // Title
            Constraint::Min(size.height.saturating_sub(8)), // Process list
            Constraint::Length(5),    // Input and feedback area
        ])
        .split(size);

    // Title
    let title = Paragraph::new("Process Control Menu")
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Process list
    let processes = app.process_manager.get_processes();
    let headers = ["PID", "NAME", "STATUS", "CPU%", "MEM(MB)", "USER"];
    
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
    
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1);

    let rows: Vec<Row> = processes
        .iter()
        .skip(app.scroll_offset)
        .take(app.display_limit)
        .enumerate()
        .map(|(i, process)| {
            let idx = app.scroll_offset + i;
            let highlight = idx == app.selected_process_index;
            let style = if highlight {
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if i % 2 == 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Blue)
            };

            let memory_mb = process.memory_usage / (1024 * 1024);

            Row::new(vec![
                Cell::from(process.pid.to_string()).style(style),
                Cell::from(process.name.clone()).style(Style::default().fg(Color::Green)),
                Cell::from(process.status.trim()).style(get_status_style(&process.status)),
                Cell::from(format!("{:.1}%", process.cpu_usage)).style(style),
                Cell::from(format!("{}", memory_mb)).style(style),
                Cell::from(process.user.clone().unwrap_or_default()).style(Style::default().fg(Color::Magenta)),
            ])
        })
        .collect();

    let process_table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Processes (↑↓ to move, Enter to select)"))
        .widths(&[
            Constraint::Length(8),   // PID
            Constraint::Length(20),  // NAME
            Constraint::Length(10),  // STATUS
            Constraint::Length(8),   // CPU%
            Constraint::Length(10),  // MEM(MB)
            Constraint::Length(12),  // USER
        ]);
    f.render_widget(process_table, chunks[1]);

    // Input and feedback area
    let input_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Commands
            Constraint::Length(3),  // Input and feedback
        ])
        .split(chunks[2]);

    // Commands help
    let commands = match app.kill_stop_input_state {
        KillStopInputState::SelectingPid => vec![
            Span::styled("Commands: ", Style::default().fg(Color::White)),
            Span::styled("[↑/↓] Move  ", Style::default().fg(Color::Cyan)),
            Span::styled("[Enter] Select  ", Style::default().fg(Color::Green)),
            Span::styled("[Esc] Back", Style::default().fg(Color::Blue)),
        ],
        KillStopInputState::EnteringAction => vec![
            Span::styled("Commands: ", Style::default().fg(Color::White)),
            Span::styled("[k] Kill  ", Style::default().fg(Color::Red)),
            Span::styled("[s] Stop  ", Style::default().fg(Color::Yellow)),
            Span::styled("[c] Continue  ", Style::default().fg(Color::Green)),
            Span::styled("[Esc] Back", Style::default().fg(Color::Blue)),
        ],
    };
    let commands_text = Paragraph::new(Line::from(commands))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(commands_text, input_area[0]);

    // Input and feedback
    let content = if let Some((msg, is_error)) = &app.input_state.message {
        // Show feedback message
        vec![
            Line::from(vec![
                Span::styled(
                    msg,
                    if *is_error {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Green)
                    }
                )
            ])
        ]
    } else if app.kill_stop_input_state == KillStopInputState::EnteringAction {
        // Show action input prompt
        if let Some(process) = processes.get(app.selected_process_index) {
            vec![
                Line::from(vec![
                    Span::styled(format!("Selected process {} (PID: {}). Enter action (k/s/c): ", process.name, process.pid), Style::default().fg(Color::Yellow)),
                    Span::styled(&app.input_state.pid_input, Style::default().fg(Color::White)),
                    Span::styled(" █", Style::default().fg(Color::White)),
                ])
            ]
        } else {
            vec![Line::from("No process selected.")]
        }
    } else {
        // Show process selection prompt
        if let Some(process) = processes.get(app.selected_process_index) {
            vec![
                Line::from(vec![
                    Span::styled(format!("Selected process {} (PID: {}). Press Enter to select action.", process.name, process.pid), Style::default().fg(Color::Yellow)),
                ])
            ]
        } else {
            vec![Line::from("No process selected.")]
        }
    };

    let input_widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Input/Feedback"))
        .alignment(Alignment::Left);
    f.render_widget(input_widget, input_area[1]);
}

fn draw_change_nice_menu(f: &mut Frame, app: &App) {
    let size = f.size();
    // Add a visually prominent title box at the top
    let title_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Make the title box taller
            Constraint::Min(1),
        ])
        .split(size);
    let title = Paragraph::new("Change Nice Value")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(ratatui::widgets::BorderType::Thick));
    f.render_widget(title, title_chunk[0]);
    let size = title_chunk[1];
    // Add a blank line below the title for spacing
    let spacing_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(size);
    let size = spacing_chunk[1];

    let process_table_width = (size.width as f32 * 0.55) as u16;
    let right_panel_width = size.width - process_table_width;
    let process_table_height = size.height - 2;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(process_table_width),
            Constraint::Length(right_panel_width),
        ])
        .split(size);

    // --- LEFT: Process Table with highlight ---
    let processes = app.process_manager.get_processes();
    let headers = ["PID", "NAME", "NICE", "CPU%", "USER"];
    let header_cells = headers
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::Blue))
        .height(1);

    let visible_processes = processes
        .iter()
        .skip(app.change_nice_scroll_offset)
        .take(process_table_height as usize - 2)
        .enumerate()
        .map(|(i, process)| {
            let idx = app.change_nice_scroll_offset + i;
            let highlight = idx == app.selected_process_index;
            let style = if highlight {
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if i % 2 == 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Blue)
            };
            Row::new(vec![
                Cell::from(process.pid.to_string()).style(style),
                Cell::from(process.name.clone()).style(Style::default().fg(Color::Green)),
                Cell::from(process.nice.to_string()).style(Style::default().fg(Color::Yellow)),
                Cell::from(format!("{:.1}%", process.cpu_usage)).style(style),
                Cell::from(process.user.clone().unwrap_or_default()).style(Style::default().fg(Color::Magenta)),
            ])
        })
        .collect::<Vec<_>>();

    let process_table = Table::new(visible_processes)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Processes (↑↓ to move, Enter to select)"))
        .widths(&[
            Constraint::Length(8),   // PID
            Constraint::Length(20),  // NAME
            Constraint::Length(8),   // NICE
            Constraint::Length(8),   // CPU%
            Constraint::Length(12),  // USER
        ]);
    f.render_widget(process_table, chunks[0]);

    // --- RIGHT: Details, Input, Instructions, Status ---
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Process details
            Constraint::Length(5), // Input box
            Constraint::Min(3),    // Instructions & status
        ])
        .split(chunks[1]);

    // Process details
    let selected = app.selected_process_index.min(processes.len().saturating_sub(1));
    let proc = processes.get(selected);
    let details = if let Some(proc) = proc {
        vec![
            Line::from(vec![Span::styled("Selected Process:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::raw(format!("PID: {}", proc.pid))]),
            Line::from(vec![Span::raw(format!("Name: {}", proc.name))]),
            Line::from(vec![Span::raw(format!("User: {}", proc.user.clone().unwrap_or_default()))]),
            Line::from(vec![Span::raw(format!("Current Nice: {}", proc.nice))]),
        ]
    } else {
        vec![Line::from("No process selected.")]
    };
    let details_box = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Details"));
    f.render_widget(details_box, right_chunks[0]);

    // Input box for nice value
    let input_text = if app.nice_input_state == NiceInputState::EnteringNice {
        format!("New nice value (-20 to 19): {}", app.input_state.nice_input)
    } else {
        "Press Enter to change nice value".to_string()
    };
    // If in selection mode or after a message, use yellow (neutral) for input box
    let input_style = if app.nice_input_state == NiceInputState::SelectingPid {
        Style::default().fg(Color::Yellow)
    } else if let Some((_, is_error)) = &app.input_state.message {
        if *is_error {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        }
    } else {
        Style::default().fg(Color::Yellow)
    };
    let input_box = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default().borders(Borders::ALL).title("Nice Value Input"));
    f.render_widget(input_box, right_chunks[1]);

    // Instructions and status
    let mut info = vec![
        Line::from(vec![Span::styled(
            "Instructions:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        )]),
        Line::from(vec![Span::raw("- Use ↑/↓ to move selection in the process list.")]),
        Line::from(vec![Span::raw("- Press Enter to select a process and input a new nice value.")]),
        Line::from(vec![Span::raw("- Type the new nice value, then Enter to apply." )]),
        Line::from(vec![Span::raw("- Press Esc to cancel and return.")]),
    ];
    if let Some((msg, is_error)) = &app.input_state.message {
        info.push(Line::from(vec![Span::styled(
            msg,
            if *is_error { Style::default().fg(Color::Red) } else { Style::default().fg(Color::Green) }
        )]));
    }
    let info_box = Paragraph::new(info)
        .block(Block::default().borders(Borders::ALL).title("Help & Status"));
    f.render_widget(info_box, right_chunks[2]);
}

fn get_status_style(status: &str) -> Style {
    match status.trim().to_lowercase().as_str() {
        "running" => Style::default().fg(Color::Green),
        "sleeping" => Style::default().fg(Color::Blue),
        "stopped" => Style::default().fg(Color::Yellow),
        "zombie" => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::White),
    }
}

fn handle_events(app: &mut App) -> Result<bool, Box<dyn Error>> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match app.view_mode {
                ViewMode::ProcessList => {
                    if handle_process_list_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::Statistics => {
                    if handle_statistics_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::FilterSort => {
                    if handle_filter_sort_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::Sort => {
                    if handle_sort_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::Filter => {
                    if handle_filter_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::FilterInput => {
                    if handle_filter_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::KillStop => {
                    if handle_kill_stop_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::ChangeNice => {
                    if handle_change_nice_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::PerProcessGraph => {
                    if handle_per_process_graph_input(key, app)? {
                        return Ok(true);
                    }
                }
                ViewMode::ProcessLog => {
                    // Handle process log input
                    return Ok(false);
                }
                ViewMode::Help => {
                    // Handle help input
                    return Ok(false);
                }
            }
        }
    }
    Ok(false)
}

fn handle_process_list_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    match key.code {
        KeyCode::Char('a') => {
            app.sort_ascending = !app.sort_ascending;
            if let Some(mode) = &app.sort_mode {
                app.process_manager.set_sort(mode, app.sort_ascending);
            }
        }        
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('s') | KeyCode::Char('S') => app.view_mode = ViewMode::Statistics,
        KeyCode::Up => {
            if app.scroll_offset > 0 {
                app.scroll_offset -= 1;
            }
        }
        KeyCode::Down => {
            let process_len = app.process_manager.get_processes().len();
            if app.scroll_offset < process_len.saturating_sub(app.display_limit) {
                app.scroll_offset += 1;
            }
        }
        KeyCode::Char('1') => app.view_mode = ViewMode::FilterSort,
        KeyCode::Char('2') => app.view_mode = ViewMode::ChangeNice,
        KeyCode::Char('3') => app.view_mode = ViewMode::KillStop,
        KeyCode::Char('4') => {
            app.view_mode = ViewMode::PerProcessGraph;
            app.selected_process_index = 0;
            app.per_process_graph_scroll_offset = 0;
            app.selected_process_for_graph = None;
        }
        KeyCode::Char('5') => app.view_mode = ViewMode::ProcessLog,
        KeyCode::Char('6') => app.view_mode = ViewMode::Help,
        _ => {}
    }
    Ok(false)
}

fn handle_statistics_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('S') => {
            app.view_mode = ViewMode::ProcessList;
            app.stats_scroll_offset = 0;  // Reset scroll when leaving statistics view
            app.current_stats_tab = StatisticsTab::Graphs;  // Reset to default tab
        }
        KeyCode::Char('1') => {
            app.current_stats_tab = StatisticsTab::Graphs;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('2') => {
            app.current_stats_tab = StatisticsTab::Overview;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('3') => {
            app.current_stats_tab = StatisticsTab::CPU;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('4') => {
            app.current_stats_tab = StatisticsTab::Memory;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('5') => {
            app.current_stats_tab = StatisticsTab::Disk;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('6') => {
            app.current_stats_tab = StatisticsTab::Processes;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('7') => {
            app.current_stats_tab = StatisticsTab::Advanced;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Char('8') => {
            app.current_stats_tab = StatisticsTab::Help;
            app.stats_scroll_offset = 0;  // Reset scroll when switching tabs
        }
        KeyCode::Up => {
            if app.current_stats_tab == StatisticsTab::CPU {
                // Smooth scrolling - move up by 1/4 of the viewport
                let scroll_amount = 3;
                app.stats_scroll_offset = app.stats_scroll_offset.saturating_sub(scroll_amount);
            }
        }
        KeyCode::Down => {
            if app.current_stats_tab == StatisticsTab::CPU {
                // Smooth scrolling - move down by 1/4 of the viewport
                let scroll_amount = 3;
                app.stats_scroll_offset = app.stats_scroll_offset.saturating_add(scroll_amount);
            }
        }
        KeyCode::PageUp => {
            if app.current_stats_tab == StatisticsTab::CPU {
                // Page up - move by half the viewport
                let scroll_amount = 10;
                app.stats_scroll_offset = app.stats_scroll_offset.saturating_sub(scroll_amount);
            }
        }
        KeyCode::PageDown => {
            if app.current_stats_tab == StatisticsTab::CPU {
                // Page down - move by half the viewport
                let scroll_amount = 10;
                app.stats_scroll_offset = app.stats_scroll_offset.saturating_add(scroll_amount);
        }
        }
        KeyCode::Home => {
            if app.current_stats_tab == StatisticsTab::CPU {
                // Jump to top
                app.stats_scroll_offset = 0;
            }
        }
        KeyCode::End => {
            if app.current_stats_tab == StatisticsTab::CPU {
                // Jump to bottom (will be bounded by max_scroll in the render function)
                app.stats_scroll_offset = usize::MAX;
            }
        }
        _ => {}
    }
    Ok(false)
}

fn handle_filter_sort_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    match key.code {
        KeyCode::Char('1') => app.view_mode = ViewMode::Sort,
        KeyCode::Char('2') => app.view_mode = ViewMode::Filter,
        KeyCode::Backspace | KeyCode::Esc => app.view_mode = ViewMode::ProcessList,
        _ => {}
    }
    Ok(false)
}

fn handle_sort_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    match key.code {
        KeyCode::Char('1') => {
            app.sort_mode = Some("pid".to_string());
            app.process_manager.set_sort("pid", app.sort_ascending);
            app.view_mode = ViewMode::ProcessList;
        }
        KeyCode::Char('2') => {
            app.sort_mode = Some("mem".to_string());
            app.process_manager.set_sort("mem", app.sort_ascending);
            app.view_mode = ViewMode::ProcessList;
        }
        KeyCode::Char('3') => {
            app.sort_mode = Some("ppid".to_string());
            app.process_manager.set_sort("ppid", app.sort_ascending);
            app.view_mode = ViewMode::ProcessList;
        }
        KeyCode::Char('4') => {
            app.sort_mode = Some("start".to_string());
            app.process_manager.set_sort("start", app.sort_ascending);
            app.view_mode = ViewMode::ProcessList;
        }
        KeyCode::Char('5') => {
            app.sort_mode = Some("nice".to_string());
            app.process_manager.set_sort("nice", app.sort_ascending);
            app.view_mode = ViewMode::ProcessList;
        }
        KeyCode::Char('6') => {
            app.sort_mode = Some("cpu".to_string());
            app.process_manager.set_sort("cpu", app.sort_ascending);
            app.view_mode = ViewMode::ProcessList;
        }
        KeyCode::Char('a') => {
            app.sort_ascending = !app.sort_ascending;
            if let Some(mode) = &app.sort_mode {
                app.process_manager.set_sort(mode, app.sort_ascending);
            }
        }
        KeyCode::Backspace | KeyCode::Esc => app.view_mode = ViewMode::FilterSort,
        _ => {}
    }
    Ok(false)
}

fn handle_filter_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    match app.view_mode {
        ViewMode::Filter => {
            match key.code {
                KeyCode::Char('1') => {
                    app.filter_mode = Some("user".to_string());
                    app.input_state.filter_input.clear();
                    app.view_mode = ViewMode::FilterInput;
                }
                KeyCode::Char('2') => {
                    app.filter_mode = Some("name".to_string());
                    app.input_state.filter_input.clear();
                    app.view_mode = ViewMode::FilterInput;
                }
                KeyCode::Char('3') => {
                    app.filter_mode = Some("pid".to_string());
                    app.input_state.filter_input.clear();
                    app.view_mode = ViewMode::FilterInput;
                }
                KeyCode::Char('4') => {
                    app.filter_mode = Some("ppid".to_string());
                    app.input_state.filter_input.clear();
                    app.view_mode = ViewMode::FilterInput;
                }
                KeyCode::Esc => {
                    app.filter_mode = None;
                    app.input_state.filter_input.clear();
                    app.process_manager.set_filter(None, None);
                    app.view_mode = ViewMode::ProcessList;
                }
                KeyCode::Backspace | KeyCode::Left => {
                    app.view_mode = ViewMode::FilterSort;
                }
                _ => {}
            }
        }
        ViewMode::FilterInput => {
            match key.code {
                KeyCode::Char(c) => {
                    if let Some(mode) = &app.filter_mode {
                        // Only allow digits for PID and PPID filters
                        if (mode == "pid" || mode == "ppid") && !c.is_ascii_digit() {
                            return Ok(false);
                        }
                        app.input_state.filter_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    app.input_state.filter_input.pop();
                }
                KeyCode::Enter => {
                    if !app.input_state.filter_input.is_empty() {
                        app.process_manager.set_filter(
                            app.filter_mode.clone(),
                            Some(app.input_state.filter_input.clone())
                        );
                        app.view_mode = ViewMode::ProcessList;
                    }
                }
                KeyCode::Left => {
                    app.view_mode = ViewMode::Filter;
                    app.input_state.filter_input.clear();
                }
                KeyCode::Esc => {
                    app.filter_mode = None;
                    app.input_state.filter_input.clear();
                    app.process_manager.set_filter(None, None);
                    app.view_mode = ViewMode::ProcessList;
                }
                _ => {}
            }
        }
        _ => unreachable!(),
    }
    Ok(false)
}

fn handle_kill_stop_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    let processes = app.process_manager.get_processes();
    match app.kill_stop_input_state {
        KillStopInputState::SelectingPid => {
            match key.code {
                KeyCode::Up => {
                    if app.selected_process_index > 0 {
                        app.selected_process_index -= 1;
                        if app.selected_process_index < app.scroll_offset {
                            app.scroll_offset = app.selected_process_index;
                        }
                    }
                }
                KeyCode::Down => {
                    if app.selected_process_index + 1 < processes.len() {
                        app.selected_process_index += 1;
                        let bottom = app.scroll_offset + app.display_limit;
                        if app.selected_process_index >= bottom {
                            app.scroll_offset = app.selected_process_index - app.display_limit + 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    if !processes.is_empty() {
                        app.kill_stop_input_state = KillStopInputState::EnteringAction;
                        app.input_state.pid_input.clear();
                        app.input_state.message = None;
                    }
                }
                KeyCode::Esc => {
                    app.view_mode = ViewMode::ProcessList;
                    app.input_state = InputState::default();
                    app.kill_stop_input_state = KillStopInputState::SelectingPid;
                }
                _ => {}
            }
        }
        KillStopInputState::EnteringAction => {
            match key.code {
                KeyCode::Char('k') | KeyCode::Char('s') | KeyCode::Char('c') => {
                    if let Some(process) = processes.get(app.selected_process_index) {
                        let action = match key.code {
                            KeyCode::Char('k') => {
                                match app.process_manager.kill_process(process.pid) {
                                    Ok(_) => Some(("Successfully killed process".to_string(), false)),
                                    Err(e) => Some((format!("Error killing process: {}", e), true)),
                                }
                            }
                            KeyCode::Char('s') => {
                                match app.process_manager.stop_process(process.pid) {
                                    Ok(_) => Some(("Successfully stopped process".to_string(), false)),
                                    Err(e) => Some((format!("Error stopping process: {}", e), true)),
                                }
                            }
                            KeyCode::Char('c') => {
                                match app.process_manager.continue_process(process.pid) {
                                    Ok(_) => Some(("Successfully continued process".to_string(), false)),
                                    Err(e) => Some((format!("Error continuing process: {}", e), true)),
                                }
                            }
                            _ => None,
                        };

                        if let Some((msg, is_error)) = action {
                            app.input_state.message = Some((
                                format!("{} {}", msg, process.pid),
                                is_error
                            ));
                            app.input_state.message_timeout = Some(std::time::Instant::now() + Duration::from_secs(1));
                            app.kill_stop_input_state = KillStopInputState::SelectingPid;
                        }
                    }
                }
                KeyCode::Esc => {
                    app.kill_stop_input_state = KillStopInputState::SelectingPid;
                    app.input_state.pid_input.clear();
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

fn handle_change_nice_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    let processes = app.process_manager.get_processes();
    match app.nice_input_state {
        NiceInputState::SelectingPid => {
            match key.code {
                KeyCode::Up => {
                    if app.selected_process_index > 0 {
                        app.selected_process_index -= 1;
                        if app.selected_process_index < app.change_nice_scroll_offset {
                            app.change_nice_scroll_offset = app.selected_process_index;
                        }
                    }
                }
                KeyCode::Down => {
                    if app.selected_process_index + 1 < processes.len() {
                        app.selected_process_index += 1;
                        let bottom = app.change_nice_scroll_offset + (PROCESS_TABLE_HEIGHT - 2);
                        if app.selected_process_index >= bottom {
                            app.change_nice_scroll_offset += 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    if !processes.is_empty() {
                        app.nice_input_state = NiceInputState::EnteringNice;
                        app.input_state.nice_input.clear();
                        app.input_state.message = None;
                    }
                }
                KeyCode::Esc => {
                    app.view_mode = ViewMode::ProcessList;
                    app.input_state = InputState::default();
                    app.nice_input_state = NiceInputState::SelectingPid;
                }
                _ => {}
            }
        }
        NiceInputState::EnteringNice => {
            match key.code {
                KeyCode::Char(c) => {
                    if c.is_ascii_digit() || (c == '-' && app.input_state.nice_input.is_empty()) {
                        app.input_state.nice_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    app.input_state.nice_input.pop();
                }
                KeyCode::Enter => {
                    if !app.input_state.nice_input.is_empty() {
                        if let (Some(proc), Ok(nice)) = (
                            processes.get(app.selected_process_index),
                            app.input_state.nice_input.parse::<i32>(),
                        ) {
                            if nice >= -20 && nice <= 19 {
                                match app.process_manager.set_niceness(proc.pid, nice) {
                                    Ok(_) => {
                                        app.input_state.message = Some((
                                            format!("Successfully changed nice value of process {} to {}", proc.pid, nice),
                                            false
                                        ));
                                        app.input_state.message_timeout = Some(std::time::Instant::now() + Duration::from_secs(1));
                                        app.nice_input_state = NiceInputState::SelectingPid;
                                        app.input_state.nice_input.clear();
                                    }
                                    Err(e) => {
                                        app.input_state.message = Some((
                                            format!("Error changing nice value: {}", e),
                                            true
                                        ));
                                        app.nice_input_state = NiceInputState::SelectingPid;
                                        app.input_state.nice_input.clear();
                                    }
                                }
                            } else {
                                app.input_state.message = Some((
                                    "Error: Nice value must be between -20 and 19".to_string(),
                                    true
                                ));
                                app.nice_input_state = NiceInputState::SelectingPid;
                                app.input_state.nice_input.clear();
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    app.nice_input_state = NiceInputState::SelectingPid;
                    app.input_state.nice_input.clear();
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

fn handle_per_process_graph_input(key: KeyEvent, app: &mut App) -> Result<bool, Box<dyn Error>> {
    let processes = app.process_manager.get_processes();
    match key.code {
        KeyCode::Char('q') => {
            app.view_mode = ViewMode::ProcessList;
            app.selected_process_for_graph = None;
            Ok(true)
        }
        KeyCode::Left => {
            // Switch to previous process
            if let Some(pid) = app.selected_process_for_graph {
                if let Some(idx) = processes.iter().position(|p| p.pid == pid) {
                    if idx > 0 {
                        app.selected_process_for_graph = Some(processes[idx - 1].pid);
                    }
                }
            }
            Ok(false)
        }
        KeyCode::Right => {
            // Switch to next process
            if let Some(pid) = app.selected_process_for_graph {
                if let Some(idx) = processes.iter().position(|p| p.pid == pid) {
                    if idx + 1 < processes.len() {
                        app.selected_process_for_graph = Some(processes[idx + 1].pid);
                    }
                }
            }
            Ok(false)
        }
        KeyCode::Up => {
            if let Some(_pid) = app.selected_process_for_graph {
                app.selected_process_for_graph = None;
            } else {
                if app.selected_process_index > 0 {
                    app.selected_process_index -= 1;
                    if app.selected_process_index < app.per_process_graph_scroll_offset {
                        app.per_process_graph_scroll_offset = app.selected_process_index;
                    }
                }
            }
            Ok(false)
        }
        KeyCode::Down => {
            if let Some(_pid) = app.selected_process_for_graph {
                app.selected_process_for_graph = None;
            } else {
                let max_index = processes.len().saturating_sub(1);
                if app.selected_process_index < max_index {
                    app.selected_process_index += 1;
                    if app.selected_process_index >= app.per_process_graph_scroll_offset + PROCESS_TABLE_HEIGHT - 2 {
                        app.per_process_graph_scroll_offset = app.selected_process_index - (PROCESS_TABLE_HEIGHT - 3);
                    }
                }
            }
            Ok(false)
        }
        KeyCode::Enter => {
            if app.selected_process_for_graph.is_none() {
                if let Some(process) = processes.get(app.selected_process_index) {
                    app.selected_process_for_graph = Some(process.pid);
                }
            }
            Ok(false)
        }
        KeyCode::Esc => {
            if app.selected_process_for_graph.is_some() {
                app.selected_process_for_graph = None;
            } else {
                app.view_mode = ViewMode::ProcessList;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn render_per_process_graph_tab(frame: &mut ratatui::Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(5),  // Process info
            Constraint::Min(0),     // Content
            Constraint::Length(2),  // Help line
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Per-Process Graph View")
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    if let Some(pid) = app.selected_process_for_graph {
        let processes = app.process_manager.get_processes();
        if let Some(process) = processes.iter().find(|p| p.pid == pid) {
            // Process info box
            let info_lines = vec![
                Line::from(vec![Span::styled(format!("Name: {}", process.name), Style::default().fg(Color::Green))]),
                Line::from(vec![Span::styled(format!("PID: {}", process.pid), Style::default().fg(Color::Yellow)), Span::raw("  "), Span::styled(format!("User: {}", process.user.clone().unwrap_or_default()), Style::default().fg(Color::Magenta))]),
                Line::from(vec![Span::styled(format!("PPID: {}", process.parent_pid.unwrap_or(0)), Style::default().fg(Color::Cyan)), Span::raw("  "), Span::styled(format!("Status: {}", process.status), Style::default().fg(Color::White))]),
                Line::from(vec![Span::styled(format!("Start: {}", process.start_time_str), Style::default().fg(Color::White))]),
            ];
            let info_box = Paragraph::new(info_lines)
                .block(Block::default().borders(Borders::ALL).title("Process Info"));
            frame.render_widget(info_box, chunks[1]);

            // Graphs
            let graph_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),  // CPU Graph
                    Constraint::Percentage(50),  // Memory Graph
                ])
                .split(chunks[2]);

            if let Some((cpu_history, mem_history)) = app.graph_data.get_process_history(pid) {
                // Live stats for CPU
                let current_cpu = cpu_history.back().copied().unwrap_or(0.0);
                let min_cpu = cpu_history.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_cpu = cpu_history.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let avg_cpu = if !cpu_history.is_empty() {
                    cpu_history.iter().sum::<f32>() / cpu_history.len() as f32
                } else { 0.0 };
                // CPU Graph
                let cpu_data: Vec<(f64, f64)> = cpu_history.iter()
                    .enumerate()
                    .map(|(i, &usage)| (i as f64, usage as f64))
                    .collect();
                let cpu_dataset = Dataset::default()
                    .name("CPU Usage")
                    .marker(ratatui::symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&cpu_data);
                let cpu_chart = Chart::new(vec![cpu_dataset])
                    .block(Block::default()
                        .title(format!("CPU Usage for {} (PID: {}) | Now: {:.1}%  Min: {:.1}%  Max: {:.1}%  Avg: {:.1}%", process.name, pid, current_cpu, min_cpu, max_cpu, avg_cpu))
                        .borders(Borders::ALL))
                    .x_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, cpu_history.len() as f64])
                        .labels(vec![]))
                    .y_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, 100.0])
                        .labels(vec!["0%".into(), "50%".into(), "100%".into()]));
                frame.render_widget(cpu_chart, graph_chunks[0]);

                // Live stats for MEM
                let current_mem = mem_history.back().copied().unwrap_or(0) as f64 / (1024.0 * 1024.0);
                let min_mem = mem_history.iter().cloned().min().unwrap_or(0) as f64 / (1024.0 * 1024.0);
                let max_mem = mem_history.iter().cloned().max().unwrap_or(0) as f64 / (1024.0 * 1024.0);
                let avg_mem = if !mem_history.is_empty() {
                    mem_history.iter().sum::<u64>() as f64 / mem_history.len() as f64 / (1024.0 * 1024.0)
                } else { 0.0 };
                let memory_data: Vec<(f64, f64)> = mem_history.iter()
                    .enumerate()
                    .map(|(i, &usage)| (i as f64, usage as f64 / (1024.0 * 1024.0)))
                    .collect();
                let max_memory = memory_data.iter()
                    .map(|&(_, y)| y)
                    .fold(0.0, f64::max)
                    .max(1.0);
                let memory_dataset = Dataset::default()
                    .name("Memory Usage")
                    .marker(ratatui::symbols::Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Green))
                    .data(&memory_data);
                let memory_chart = Chart::new(vec![memory_dataset])
                    .block(Block::default()
                        .title(format!("Memory Usage for {} (PID: {}) | Now: {:.2} MB  Min: {:.2} MB  Max: {:.2} MB  Avg: {:.2} MB", process.name, pid, current_mem, min_mem, max_mem, avg_mem))
                        .borders(Borders::ALL))
                    .x_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, mem_history.len() as f64])
                        .labels(vec![]))
                    .y_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, max_memory * 1.2])
                        .labels(vec![
                            "0 MB".into(),
                            format!("{:.1} MB", max_memory / 2.0).into(),
                            format!("{:.1} MB", max_memory).into(),
                        ]));
                frame.render_widget(memory_chart, graph_chunks[1]);
            }
        }
        // Help line
        let help = Paragraph::new("←/→: Next/Prev process  ↑/↓: Back to list  Enter: Select  Esc: Back  Q: Quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);
    } else {
        // Show process selection list
        let processes = app.process_manager.get_processes();
        let headers = ["PID", "NAME", "CPU%", "MEM(MB)", "USER"];
        let header_cells = headers
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Blue))
            .height(1);
        let rows: Vec<Row> = processes
            .iter()
            .skip(app.per_process_graph_scroll_offset)
            .take(PROCESS_TABLE_HEIGHT - 2)
            .enumerate()
            .map(|(i, process)| {
                let idx = app.per_process_graph_scroll_offset + i;
                let highlight = idx == app.selected_process_index;
                let style = if highlight {
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if i % 2 == 0 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Blue)
                };
                let memory_mb = process.memory_usage / (1024 * 1024);
                Row::new(vec![
                    Cell::from(process.pid.to_string()).style(style),
                    Cell::from(process.name.clone()).style(Style::default().fg(Color::Green)),
                    Cell::from(format!("{:.1}%", process.cpu_usage)).style(style),
                    Cell::from(format!("{}", memory_mb)).style(style),
                    Cell::from(process.user.clone().unwrap_or_default()).style(Style::default().fg(Color::Magenta)),
                ])
            })
            .collect();
        let table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Select a Process (↑↓ to move, Enter to select, Esc to return)"))
            .widths(&[
                Constraint::Length(8),   // PID
                Constraint::Length(20),  // NAME
                Constraint::Length(8),   // CPU%
                Constraint::Length(10),  // MEM(MB)
                Constraint::Length(12),  // USER
            ]);
        frame.render_widget(table, chunks[2]);
        // Help line
        let help = Paragraph::new("↑/↓: Move  Enter: Select  Esc: Back  Q: Quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);
    }
}

fn render_process_log_tab(frame: &mut ratatui::Frame, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled("Process Log View", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("View detailed logs for each process", Style::default().fg(Color::Gray))]),
    ];
    let widget = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Process Log"));
    frame.render_widget(widget, area);
}

fn render_help_tab(frame: &mut ratatui::Frame, area: Rect) {
    let text = vec![
        Line::from(vec![Span::styled("Help & Documentation", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("Navigation:", Style::default().fg(Color::Cyan))]),
        Line::from(vec![Span::styled("↑/↓ - Scroll through processes", Style::default().fg(Color::Gray))]),
        Line::from(vec![Span::styled("1-6 - Switch between views", Style::default().fg(Color::Gray))]),
        Line::from(vec![Span::styled("S - Show statistics", Style::default().fg(Color::Gray))]),
        Line::from(vec![Span::styled("q - Quit", Style::default().fg(Color::Gray))]),
    ];
    let widget = Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Help"));
    frame.render_widget(widget, area);
}
