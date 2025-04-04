use crate::process;
use crate::process::ProcessInfo;  // ProcessInfo is defined in process.rs
use std::thread::sleep;
use std::time::Duration;
use process::ProcessManager;
use std::error::Error;
use crossterm::{
    cursor, execute, terminal, terminal::ClearType,
    event::{self, Event, KeyCode},
};
use crossterm::{
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor, Attribute, SetAttribute},
    ExecutableCommand, 
    QueueableCommand,
};
use std::io::{stdout, Write};

// Setup the terminal (raw mode + alternate screen)
pub fn setup_terminal() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(stdout(), terminal::EnterAlternateScreen)?;
    Ok(())
}


// Restore terminal back to normal
pub fn restore_terminal() -> std::io::Result<()> {
    execute!(stdout(), terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}


//ui_renderer
pub fn ui_renderer() -> Result<(), Box<dyn Error>> {
    setup_terminal()?; // Setup terminal for raw mode and alternate screen

    let mut process_manager = ProcessManager::new(); // Initialize process manager
    let mut scroll_offset: usize = 0;                // Track scrolling position
    let display_limit: usize = 20;                   // Number of processes visible at a time
    let process_len = process_manager.get_processes().len(); // Total number of processes

    loop {
        // Handle key press events
        if handle_key_event(&mut scroll_offset, display_limit, process_len)? {
            break; // Exit loop if 'q' is pressed
        }

        // Refresh and draw
        process_manager.refresh();
        let processes = process_manager.get_processes();
        draw_processes(&processes, scroll_offset, display_limit)?;
        draw_menu(display_limit)?;

        sleep(Duration::from_millis(100));
    }

    restore_terminal()?; // Reset terminal state
    Ok(())
}


pub fn handle_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Char('q') => return Ok(true), // Signal to quit
                KeyCode::Up => {
                    if *scroll_offset > 0 {
                        *scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    if *scroll_offset < process_len.saturating_sub(display_limit) {
                        *scroll_offset += 1;
                    }
                }
                KeyCode::Char('1') => {
                    draw_filter_menu()?; // Enter filter menu
                }
                _ => {}
            }
        }
    }
    Ok(false)
}
pub fn handle_filter_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {

                KeyCode::Char('1') => {
                    draw_sort_menu()?; // Enter sort menu
                }
                KeyCode::Char('2') => {
                    // TODO: Enter Filter menu
                }

                KeyCode::Backspace => return Ok(true), // Signal to quit

                KeyCode::Up => {
                    if *scroll_offset > 0 {
                        *scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    if *scroll_offset < process_len.saturating_sub(display_limit) {
                        *scroll_offset += 1;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

pub fn handle_sort_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {

                KeyCode::Char('1') => {
                    draw_sorted_processes(20, "pid")?; // Enter sorted menu
                }
                KeyCode::Char('2') => {
                    draw_sorted_processes(20, "mem")?; // Enter sorted menu
                }
                KeyCode::Char('3') => {
                    draw_sorted_processes(20, "ppid")?; // Enter sorted menu
                }
                KeyCode::Char('4') => {
                    draw_sorted_processes(20, "start")?; // Enter sorted menu
                }
                KeyCode::Backspace => return Ok(true), // Signal to quit

                KeyCode::Up => {
                    if *scroll_offset > 0 {
                        *scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    if *scroll_offset < process_len.saturating_sub(display_limit) {
                        *scroll_offset += 1;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

pub fn handle_ssort_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Backspace => return Ok(true), // Signal to quit
                KeyCode::Up => {
                    if *scroll_offset > 0 {
                        *scroll_offset -= 1;
                    }
                }
                KeyCode::Down => {
                    if *scroll_offset < process_len.saturating_sub(display_limit) {
                        *scroll_offset += 1;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(false)
}

pub fn draw_processes(processes: &[ProcessInfo], scroll_offset: usize, display_limit: usize) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    // Header in bold bright white on blue background
    stdout.execute(SetAttribute(Attribute::Bold))?;
    stdout.execute(SetForegroundColor(Color::White))?;
    stdout.execute(SetBackgroundColor(Color::Blue))?;
    
    writeln!(
        stdout,
        "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>12} {:>10}",
        "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "USER", "STATUS",
    )?;
    
    // Reset colors and styling
    stdout.execute(ResetColor)?;
    stdout.execute(SetAttribute(Attribute::Reset))?;

    // Calculate range of processes to display based on scroll_offset
    let start_index = scroll_offset;
    let end_index = (scroll_offset + display_limit).min(processes.len());

    // Print the processes in the specified range
    for (i, process) in processes.iter().enumerate().take(end_index).skip(start_index) {
        execute!(stdout, cursor::MoveTo(0, (i - start_index + 1) as u16))?;

        // Format the process name
        let name = if process.name.len() > 15 {
            format!("{:.12}...", process.name)
        } else {
            process.name.clone()
        };
        
        // Format the user field
        let user = process.user.clone().unwrap_or_default();
        let user_display = if user.len() > 10 {
            format!("{:.7}...", user)
        } else {
            user
        };

        let memory_mb = process.memory_usage / (1024 * 1024);
        
        // Set PID color based on odd/even rows for readability
        if i % 2 == 0 {
            stdout.execute(SetForegroundColor(Color::Cyan))?;
        } else {
            stdout.execute(SetForegroundColor(Color::Blue))?;
        }
        
        write!(stdout, "{:<6} ", process.pid)?;
        
        // Process name in green
        stdout.execute(SetForegroundColor(Color::Green))?;
        write!(stdout, "{:<18} ", name)?;
        
        // CPU usage with color based on value
        let cpu_color = match process.cpu_usage {
            c if c > 50.0 => Color::Red,
            c if c > 25.0 => Color::Yellow,
            _ => Color::Green,
        };
        stdout.execute(SetForegroundColor(cpu_color))?;
        write!(stdout, "{:>6.2} ", process.cpu_usage)?;
        
        // Memory usage with color based on value
        let mem_color = match memory_mb {
            m if m > 1000 => Color::Red,
            m if m > 500 => Color::Yellow,
            _ => Color::Green,
        };
        stdout.execute(SetForegroundColor(mem_color))?;
        write!(stdout, "{:>10} ", memory_mb)?;
        
        // PPID in light blue
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        write!(stdout, "{:>8} ", process.parent_pid.unwrap_or(0))?;
        
        // Start time in default color
        stdout.execute(SetForegroundColor(Color::White))?;
        write!(stdout, "{:>12} ", process.start_time)?;
        
        // User in magenta
        stdout.execute(SetForegroundColor(Color::Magenta))?;
        write!(stdout, "{:>12} ", user_display)?;
        
        // Status with color based on state
        let status = process.status.trim();
        let status_color = match status.to_lowercase().as_str() {
            "running" => Color::Green,
            "sleeping" => Color::Blue,
            "stopped" => Color::Yellow,
            "zombie" => Color::Red,
            _ => Color::White,
        };
        stdout.execute(SetForegroundColor(status_color))?;
        writeln!(stdout, "{:>10}", status)?;
    }
    
    // Reset colors
    stdout.execute(ResetColor)?;
    stdout.flush()?;
    Ok(())
}

pub fn draw_menu(display_limit: usize) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
    
    // Navigation in cyan
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    write!(stdout, "[↑] Scroll Up  |  [↓] Scroll Down")?;
    stdout.execute(ResetColor)?;
    writeln!(stdout)?;
    
    execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
    
    // Menu options with different colors
    stdout.execute(SetForegroundColor(Color::Yellow))?;
    write!(stdout, "1. Filter")?;
    stdout.execute(ResetColor)?;
    write!(stdout, "  |  ")?;
    
    stdout.execute(SetForegroundColor(Color::Green))?;
    write!(stdout, "2. Change Priority")?;
    stdout.execute(ResetColor)?;
    write!(stdout, "  |  ")?;
    
    stdout.execute(SetForegroundColor(Color::Red))?;
    write!(stdout, "3. Kill/Stop Process")?;
    stdout.execute(ResetColor)?;
    write!(stdout, " |  ")?;
    
    stdout.execute(SetForegroundColor(Color::Magenta))?;
    write!(stdout, "[Q] Quit")?;
    stdout.execute(ResetColor)?;
    writeln!(stdout)?;
    
    stdout.flush()?;
    Ok(())
}

pub fn draw_filter_menu() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new(); // Initialize process manager
    let mut scroll_offset: usize = 0;                // Track scrolling position
    let display_limit: usize = 20;                   // Number of processes visible at a time
    let process_len = process_manager.get_processes().len(); // Total number of processes

    loop {
        process_manager.refresh();
        let processes = process_manager.get_processes().clone();
        if handle_filter_key_event(&mut scroll_offset, display_limit, process_len)? {
            break; // Exit loop if 'q' is pressed
        }

        draw_processes(&processes, scroll_offset, display_limit)?;
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        
        // Menu option 1 in yellow
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "1. Sort")?;
        
        // Separator
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Menu option 2 in green
        stdout.execute(SetForegroundColor(Color::Green))?;
        write!(stdout, "2. Filter")?;
        
        // Separator
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Back button in blue
        stdout.execute(SetForegroundColor(Color::Blue))?;
        writeln!(stdout, "[←] Back")?;
        
        // Reset color
        stdout.execute(ResetColor)?;
    }

    stdout.flush()?;
    sleep(Duration::from_millis(100));
    
    Ok(())
}

pub fn draw_sort_menu() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new(); // Initialize process manager
    let mut scroll_offset: usize = 0;                // Track scrolling position
    let display_limit: usize = 20;                   // Number of processes visible at a time
    let process_len = process_manager.get_processes().len(); // Total number of processes
    
    loop {
        process_manager.refresh();
        let processes = process_manager.get_processes().clone();
        if handle_sort_key_event(&mut scroll_offset, display_limit, process_len)? {
            break; // Exit loop if 'q' is pressed
        }
        
        draw_processes(&processes, scroll_offset, display_limit)?;
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        
        // Option 1
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "1. Sort by PID")?;
        
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Option 2
        stdout.execute(SetForegroundColor(Color::Green))?;
        write!(stdout, "2. Sort by MEM")?;
        
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Option 3
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        write!(stdout, "3. Sort by PPID")?;
        
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Option 4
        stdout.execute(SetForegroundColor(Color::White))?;
        write!(stdout, "4. Sort by Start")?;
        
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Back button
        stdout.execute(SetForegroundColor(Color::Blue))?;
        writeln!(stdout, "[←] Back")?;
        
        // Reset colors
        stdout.execute(ResetColor)?;
    }
    
    stdout.flush()?;
    sleep(Duration::from_millis(100));
    Ok(())
}

    pub fn draw_sorted_processes(display_limit: usize, sort_mode: &str) -> std::io::Result<()> {
        let mut stdout = stdout();
        let mut process_manager = ProcessManager::new();
        let mut scroll_offset: usize = 0;
    
        loop {
            process_manager.refresh();
            let mut processes = process_manager.get_processes().clone();
    
            // Apply sorting based on sort_mode (logic unchanged)
            match sort_mode {
                "pid" => processes.sort_by_key(|p| p.pid),
                "ppid" => processes.sort_by_key(|p| p.parent_pid.unwrap_or(0)),
                "mem" => processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage)),
                "start" => processes.sort_by(|a, b| a.start_time.cmp(&b.start_time)),
                _ => {} // default: no sort
            }
    
            if handle_ssort_key_event(&mut scroll_offset, display_limit, processes.len())? {
                break;
            }
    
            // Clear and draw header with color
            execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
            
            // Header in bold bright white on blue background
            stdout.execute(SetAttribute(Attribute::Bold))?;
            stdout.execute(SetForegroundColor(Color::White))?;
            stdout.execute(SetBackgroundColor(Color::Blue))?;
            
            writeln!(
                stdout,
                "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>12} {:>10}",
                "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "USER", "STATUS",
            )?;
            
            // Reset colors and styling
            stdout.execute(ResetColor)?;
            stdout.execute(SetAttribute(Attribute::Reset))?;
    
            let start_index = scroll_offset;
            let end_index = (scroll_offset + display_limit).min(processes.len());
    
            for (i, process) in processes.iter().enumerate().take(end_index).skip(start_index) {
                execute!(stdout, cursor::MoveTo(0, (i - start_index + 1) as u16))?;
    
                let name = if process.name.len() > 15 {
                    format!("{:.12}...", process.name)
                } else {
                    process.name.clone()
                };
    
                let user = process.user.clone().unwrap_or_default();
                let user_display = if user.len() > 10 {
                    format!("{:.7}...", user)
                } else {
                    user
                };
    
                let memory_mb = process.memory_usage / (1024 * 1024);
                
                // Set PID color based on odd/even rows for readability
                if i % 2 == 0 {
                    stdout.execute(SetForegroundColor(Color::Cyan))?;
                } else {
                    stdout.execute(SetForegroundColor(Color::Blue))?;
                }
                
                write!(stdout, "{:<6} ", process.pid)?;
                
                // Process name in green
                stdout.execute(SetForegroundColor(Color::Green))?;
                write!(stdout, "{:<18} ", name)?;
                
                // CPU usage with color based on value
                let cpu_color = match process.cpu_usage {
                    c if c > 50.0 => Color::Red,
                    c if c > 25.0 => Color::Yellow,
                    _ => Color::Green,
                };
                stdout.execute(SetForegroundColor(cpu_color))?;
                write!(stdout, "{:>6.2} ", process.cpu_usage)?;
                
                // Memory usage with color based on value
                let mem_color = match memory_mb {
                    m if m > 1000 => Color::Red,
                    m if m > 500 => Color::Yellow,
                    _ => Color::Green,
                };
                stdout.execute(SetForegroundColor(mem_color))?;
                write!(stdout, "{:>10} ", memory_mb)?;
                
                // PPID in light blue
                stdout.execute(SetForegroundColor(Color::Cyan))?;
                write!(stdout, "{:>8} ", process.parent_pid.unwrap_or(0))?;
                
                // Start time in default color
                stdout.execute(SetForegroundColor(Color::White))?;
                write!(stdout, "{:>12} ", process.start_time)?;
                
                // User in magenta
                stdout.execute(SetForegroundColor(Color::Magenta))?;
                write!(stdout, "{:>12} ", user_display)?;
                
                // Status with color based on state
                let status = process.status.trim();
                let status_color = match status.to_lowercase().as_str() {
                    "running" => Color::Green,
                    "sleeping" => Color::Blue,
                    "stopped" => Color::Yellow,
                    "zombie" => Color::Red,
                    _ => Color::White,
                };
                stdout.execute(SetForegroundColor(status_color))?;
                writeln!(stdout, "{:>10}", status)?;
            }
    
            // Reset color before drawing navigation
            stdout.execute(ResetColor)?;
            
            execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
            
            // Navigation in cyan
            stdout.execute(SetForegroundColor(Color::Cyan))?;
            writeln!(stdout, "[↑] Scroll Up  |  [↓] Scroll Down")?;
            
            execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
            
            // Back button in blue
            stdout.execute(SetForegroundColor(Color::Blue))?;
            writeln!(stdout, "[←] Back")?;
            
            // Reset color
            stdout.execute(ResetColor)?;
            
            stdout.flush()?;
            sleep(Duration::from_millis(100));
        }
    
        Ok(())
    }