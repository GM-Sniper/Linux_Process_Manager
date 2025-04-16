use crate::process;
use std::io::{Write, stdin, stdout};
use crate::process::ProcessInfo;  // ProcessInfo is defined in process.rs
use std::thread::sleep;
use std::time::Duration;
use process::ProcessManager;
use std::error::Error;
use crossterm::{
    cursor, execute, terminal, terminal::ClearType,
    event::{self, Event, KeyCode, KeyEvent},
};
use crossterm::{
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor, Attribute, SetAttribute, Stylize},
    ExecutableCommand, 
    QueueableCommand,
};

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
                KeyCode::Char('2') => {
                    // changing niceness
                    change_process_niceness()?;
                    
                }
                KeyCode::Char('3') => {
                    // Placeholder for killing/stopping process
                    draw_kill_stop_menu()?;
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
                    // Placeholder for filtering processes

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
                KeyCode::Char('5') => {
                    draw_sorted_processes(20, "nice")?; // Enter sorted menu for nice value
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
    ascending: &mut bool,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Backspace => return Ok(true), // Signal to quit
                KeyCode::Char('a') => *ascending = !*ascending,
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

pub fn handle_kill_stop_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {

                KeyCode::Char('1') => {
                  kill_process()?; // Enter kill menu
                }
                KeyCode::Char('2') => {
                  stop_process()?; //Enter Stop menu 
                }
                KeyCode::Backspace => return Ok(true), // Signal to quit

                KeyCode::Char('q') => return Ok(true),

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

pub fn handle_kill_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {

                KeyCode::Char('1') => {
                  //  draw_kill_menu()?; // Enter kill menu
                }
                KeyCode::Char('2') => {
                //    draw_stop_menu()?; //Enter Stop menu 

                }
                KeyCode::Backspace => return Ok(true), // Signal to quit

                KeyCode::Char('q') => return Ok(true),

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

pub fn handle_stop_key_event(
    scroll_offset: &mut usize,
    display_limit: usize,
    process_len: usize
) -> std::io::Result<bool> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {

                KeyCode::Char('1') => {
                  //  draw_kill_menu()?; // Enter kill menu
                }
                KeyCode::Char('2') => {
                //    draw_stop_menu()?; //Enter Stop menu 

                }
                KeyCode::Backspace => return Ok(true), // Signal to quit

                KeyCode::Char('q') => return Ok(true),


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
        "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>8} {:>12} {:>10}",
        "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "NICE", "USER", "STATUS",
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
        
        // Start time in default color (formatted time)
        stdout.execute(SetForegroundColor(Color::White))?;
        // write!(stdout, "{:>12} ", process.start_time)?; // Old start time
        write!(stdout, "{:>12} ", process.startTime)?; // New formatted start time
        
        // Nice value in yellow
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "{:>8} ", process.nice)?;
        
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
    write!(stdout, "1. Sort and Filter")?;
    stdout.execute(ResetColor)?;
    write!(stdout, "  |  ")?;
    
    stdout.execute(SetForegroundColor(Color::Green))?;
    write!(stdout, "2. Change Niceness")?;
    stdout.execute(ResetColor)?;
    write!(stdout, "  |  ")?;
    
    stdout.execute(SetForegroundColor(Color::Red))?;
    write!(stdout, "3. Kill/Stop Process")?;
    stdout.execute(ResetColor)?;
    write!(stdout, " |  ")?;
    stdout.execute(SetForegroundColor(Color::Cyan))?;
    write!(stdout, "4. Search for a Process")?;
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
        
        // Option 5
        stdout.execute(SetForegroundColor(Color::Magenta))?;
        write!(stdout, "5. Sort by Nice")?;
        
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
    let mut ascending = true;


    loop {
        process_manager.refresh();
        let mut processes = process_manager.get_processes().clone();

        // Apply sorting based on sort_mode
        match sort_mode {
            "pid" => {
                if ascending {
                    processes.sort_by_key(|p| p.pid);
                } else {
                    processes.sort_by_key(|p| std::cmp::Reverse(p.pid));
                }
            }
            "ppid" => {
                if ascending {
                    processes.sort_by_key(|p| p.parent_pid.unwrap_or(0));
                } else {
                    processes.sort_by_key(|p| std::cmp::Reverse(p.parent_pid.unwrap_or(0)));
                }
            }
            "mem" => {
                if ascending {
                    processes.sort_by(|a, b| a.memory_usage.cmp(&b.memory_usage));
                } else {
                    processes.sort_by(|a, b| b.memory_usage.cmp(&a.memory_usage));
                }
            }
            "start" => {
                if ascending {
                    processes.sort_by(|a, b| a.start_time.cmp(&b.start_time));
                } else {
                    processes.sort_by(|a, b| b.start_time.cmp(&a.start_time));
                }
            }
            "nice" => {
                if ascending {
                    processes.sort_by_key(|p| p.nice);
                } else {
                    processes.sort_by_key(|p| std::cmp::Reverse(p.nice));
                }
            }
            _ => {}
        }

        if handle_ssort_key_event(&mut scroll_offset, display_limit, &mut ascending, processes.len())? {
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
            "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>8} {:>12} {:>10}",
            "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "NICE", "USER", "STATUS",
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
            write!(stdout, "{:>12} ", process.startTime)?;
            
            // Nice value in yellow
            stdout.execute(SetForegroundColor(Color::Yellow))?;
            write!(stdout, "{:>8} ", process.nice)?;
            
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


fn change_process_niceness() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new();
    let mut scroll_offset: usize = 0;
    let display_limit: usize = 20;
    
    // Input state variables
    let mut pid_input = String::new();
    let mut input_mode = true; // true = collecting PID, false = collecting nice value
    let mut nice_input = String::new();
    let mut message = String::new();
    let mut message_is_error = false;
    let mut show_message_until = std::time::Instant::now();
    
    loop {
        // Refresh process list
        process_manager.refresh();
        let processes = process_manager.get_processes();
        
        // Draw processes
        draw_processes(&processes, scroll_offset, display_limit)?;
        
        // Draw input prompts - always show both prompts
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
        
        // PID input prompt (always shown)
        stdout.execute(SetForegroundColor(Color::Green))?;
        write!(stdout, "Enter PID to change niceness (or 'q' to quit): ")?;
        
        if input_mode {
            // Highlight active input field
            stdout.execute(SetForegroundColor(Color::White))?;
            stdout.execute(SetAttribute(Attribute::Bold))?;
        } else {
            // Dim inactive input field
            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
        }
        
        write!(stdout, "{}", pid_input)?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        
        // Nice value prompt (always shown on next line)
        execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
        stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
        
        stdout.execute(SetForegroundColor(Color::Green))?;
        write!(stdout, "Enter new nice value (0-19, or -20 to -1 for root), or 'q' to cancel: ")?;
        
        if !input_mode {
            // Highlight active input field
            stdout.execute(SetForegroundColor(Color::White))?;
            stdout.execute(SetAttribute(Attribute::Bold))?;
        } else {
            // Dim inactive input field
            stdout.execute(SetForegroundColor(Color::DarkGrey))?;
        }
        
        write!(stdout, "{}", nice_input)?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        
        // Show message if needed
        if std::time::Instant::now() < show_message_until {
            execute!(stdout, cursor::MoveTo(0, (display_limit + 4) as u16))?;
            stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
            
            if message_is_error {
                // Error message in red
                stdout.execute(SetForegroundColor(Color::Red))?;
            } else {
                // Success message in green
                stdout.execute(SetForegroundColor(Color::Green))?;
            }
            
            write!(stdout, "{}", message)?;
            stdout.execute(SetForegroundColor(Color::Reset))?;
        }
        
        stdout.flush()?;
        
        // Handle input events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // Handle quit
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                    if input_mode {
                        return Ok(());  // Return to main menu
                    } else {
                        // Return to PID input mode
                        input_mode = true;
                        nice_input.clear();
                    }
                },
                
                // Handle scrolling
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    if scroll_offset > 0 {
                        scroll_offset -= 1;
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    if scroll_offset < processes.len().saturating_sub(display_limit) {
                        scroll_offset += 1;
                    }
                },
                
                // Handle backspace
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    if input_mode {
                        pid_input.pop();
                    } else {
                        nice_input.pop();
                    }
                },
                
                // Handle enter
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    if input_mode {
                        // Process PID input
                        match pid_input.trim().parse::<u32>() {
                            Ok(_pid) => {
                                // Switch to nice value input mode
                                input_mode = false;
                            },
                            Err(_) => {
                                message = "Invalid PID format. Please try again.".to_string();
                                message_is_error = true;
                                show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                                pid_input.clear();
                            }
                        }
                    } else {
                        // Process nice value input
                        match nice_input.trim().parse::<i32>() {
                            Ok(nice) if nice >= -20 && nice <= 19 => {
                                let pid = pid_input.trim().parse::<u32>().unwrap(); // Safe because we validated earlier
                                
                                // Try to set niceness
                                match process_manager.set_niceness(pid, nice) {
                                    Ok(_) => {
                                        message = format!("Successfully changed niceness of process {} to {}", pid, nice);
                                        message_is_error = false;
                                    },
                                    Err(e) => {
                                        message = format!("Error: {}", e);
                                        message_is_error = true;
                                    }
                                }
                                
                                show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                                pid_input.clear();
                                nice_input.clear();
                                input_mode = true; // Return to PID input mode
                            },
                            _ => {
                                message = "Invalid nice value. Must be between -20 and 19.".to_string();
                                message_is_error = true;
                                show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                                nice_input.clear();
                            }
                        }
                    }
                },
                
                // Handle character input
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                    if input_mode {
                        pid_input.push(c);
                    } else {
                        nice_input.push(c);
                    }
                },
                
                _ => {}
            }
        }
    }
}


pub fn draw_kill_stop_menu() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new(); // Initialize process manager
    let mut scroll_offset: usize = 0;                // Track scrolling position
    let display_limit: usize = 20;                   // Number of processes visible at a time
    let process_len = process_manager.get_processes().len(); // Total number of processes

    loop {
        process_manager.refresh();
        let processes = process_manager.get_processes().clone();
        if handle_kill_stop_key_event(&mut scroll_offset, display_limit, process_len)? 
        {
            break; // Exit loop if 'back space' is pressed
        }

        draw_processes(&processes, scroll_offset, display_limit)?;
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        
        // Menu option 1 in yellow
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "1. Kill")?;
        
        // Separator
        stdout.execute(ResetColor)?;
        write!(stdout, "  |  ")?;
        
        // Menu option 2 in green
        stdout.execute(SetForegroundColor(Color::Green))?;
        write!(stdout, "2. Stop")?;
        
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




fn stop_process() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new();
    let mut scroll_offset: usize = 0;
    let display_limit: usize = 20;
    
    // Input state variables
    let mut pid_input = String::new();
    let mut message = String::new();
    let mut message_is_error = false;
    let mut show_message_until = std::time::Instant::now();
    
    loop {
        // Refresh process list
        process_manager.refresh();
        let processes = process_manager.get_processes();
        
        // Draw processes
        draw_processes(&processes, scroll_offset, display_limit)?;
        
        // Draw input prompt
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
        
        // PID input prompt
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        write!(stdout, "Enter PID to stop (or 'q' to quit): ")?;
        
        // Highlight input field
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(SetAttribute(Attribute::Bold))?;
        write!(stdout, "{}", pid_input)?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        
        // Show message if needed
        if std::time::Instant::now() < show_message_until {
            execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
            stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
            
            if message_is_error {
                // Error message in red
                stdout.execute(SetForegroundColor(Color::Red))?;
            } else {
                // Success message in green
                stdout.execute(SetForegroundColor(Color::Green))?;
            }
            
            write!(stdout, "{}", message)?;
            stdout.execute(SetForegroundColor(Color::Reset))?;
        }
        
        stdout.flush()?;
        
        // Handle input events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // Handle quit
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                    return Ok(());  // Return to main menu
                },
                
                // Handle scrolling
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    if scroll_offset > 0 {
                        scroll_offset -= 1;
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    if scroll_offset < processes.len().saturating_sub(display_limit) {
                        scroll_offset += 1;
                    }
                },
                
                // Handle backspace
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    pid_input.pop();
                },
                
                // Handle enter
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    match pid_input.trim().parse::<u32>() {
                        Ok(pid) => {
                            // Try to stop the process
                            match process_manager.stop_process(pid) {
                                Ok(_) => {
                                    message = format!("Successfully stopped process {}", pid);
                                    message_is_error = false;
                                },
                                Err(e) => {
                                    message = format!("Error: {}", e);
                                    message_is_error = true;
                                }
                            }
                            
                            show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                            pid_input.clear();
                        },
                        Err(_) => {
                            message = "Invalid PID format. Please try again.".to_string();
                            message_is_error = true;
                            show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                            pid_input.clear();
                        }
                    }
                },
                
                // Handle character input
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                    pid_input.push(c);
                },
                
                _ => {}
            }
        }
    }
}



fn kill_process() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new();
    let mut scroll_offset: usize = 0;
    let display_limit: usize = 20;
    
    // Input state variables
    let mut pid_input = String::new();
    let mut message = String::new();
    let mut message_is_error = false;
    let mut show_message_until = std::time::Instant::now();
    
    loop {
        // Refresh process list
        process_manager.refresh();
        let processes = process_manager.get_processes();
        
        // Draw processes
        draw_processes(&processes, scroll_offset, display_limit)?;
        
        // Draw input prompt
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
        
        // PID input prompt
        stdout.execute(SetForegroundColor(Color::Red))?;
        write!(stdout, "Enter PID to kill (or 'q' to quit): ")?;
        
        // Highlight input field
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(SetAttribute(Attribute::Bold))?;
        write!(stdout, "{}", pid_input)?;
        stdout.execute(SetAttribute(Attribute::Reset))?;
        
        // Show message if needed
        if std::time::Instant::now() < show_message_until {
            execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
            stdout.execute(terminal::Clear(ClearType::CurrentLine))?;
            
            if message_is_error {
                // Error message in red
                stdout.execute(SetForegroundColor(Color::Red))?;
            } else {
                // Success message in green
                stdout.execute(SetForegroundColor(Color::Green))?;
            }
            
            write!(stdout, "{}", message)?;
            stdout.execute(SetForegroundColor(Color::Reset))?;
        }
        
        stdout.flush()?;
        
        // Handle input events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                // Handle quit
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                    return Ok(());  // Return to main menu
                },
                
                // Handle scrolling
                Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                    if scroll_offset > 0 {
                        scroll_offset -= 1;
                    }
                },
                Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                    if scroll_offset < processes.len().saturating_sub(display_limit) {
                        scroll_offset += 1;
                    }
                },
                
                // Handle backspace
                Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                    pid_input.pop();
                },
                
                // Handle enter
                Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    match pid_input.trim().parse::<u32>() {
                        Ok(pid) => {
                            // Try to kill the process
                            match process_manager.kill_process(pid) {
                                Ok(_) => {
                                    message = format!("Successfully killed process {}", pid);
                                    message_is_error = false;
                                },
                                Err(e) => {
                                    message = format!("Error: {}", e);
                                    message_is_error = true;
                                }
                            }
                            
                            show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                            pid_input.clear();
                        },
                        Err(_) => {
                            message = "Invalid PID format. Please try again.".to_string();
                            message_is_error = true;
                            show_message_until = std::time::Instant::now() + Duration::from_secs(2);
                            pid_input.clear();
                        }
                    }
                },
                
                // Handle character input
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                    pid_input.push(c);
                },
                
                _ => {}
            }
        }
    }
}
