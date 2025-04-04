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

// Draw the process list
pub fn draw_processes(processes: &[ProcessInfo], scroll_offset: usize, display_limit: usize) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    writeln!(
        stdout,
        "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>12} {:>10}",
        "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "USER", "STATUS",
    )?;

    // Calculate range of processes to display based on scroll_offset
    let start_index = scroll_offset;
    let end_index = (scroll_offset + display_limit).min(processes.len());

    // Print the processes in the specified range
    // This is where the processes are displayed
    // The processes are displayed in a table format
    for (i, process) in processes.iter().enumerate().take(end_index).skip(start_index) {
        // Move the cursor to the correct position for each process (start of each line)
        
        execute!(stdout, cursor::MoveTo(0, (i - start_index + 1) as u16))?;
        

        // Format the process name to fit within 15 characters
        // If the name is longer than 15 characters, truncate it and add "..."
        // If the name is shorter than 15 characters, display it as is
        let name = if process.name.len() > 15 {
            format!("{:.12}...", process.name)
        } else {
            process.name.clone()
        };
        // Format the user field (max 10 chars)
        let user = process.user.clone().unwrap_or_default();
        let user_display = if user.len() > 10 {
            format!("{:.7}...", user) // Keep first 7 characters and add "..."
        } else {
            user
        };



        let memory_mb = process.memory_usage / (1024 * 1024);

        // Print the process information in a formatted manner
        writeln!(
            stdout,
            "{:<6} {:<18} {:>6.2} {:>10} {:>8} {:>12} {:>12} {:>10}",
            process.pid,
            name,
            process.cpu_usage,
            memory_mb,
            process.parent_pid.unwrap_or(0),
            process.start_time,
            user_display,
            process.status.trim(), // Trim any unwanted spaces or newlines
        )?;
    }

    stdout.flush()?;
    Ok(())
}


// Draw the menu options
pub fn draw_menu(display_limit: usize) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
    writeln!(stdout, "[↑] Scroll Up  |  [↓] Scroll Down \n")?;
    execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
    writeln!(stdout, "1. Filter  |  2. Change Priority  |  3. Kill/Stop Process |  [Q] Quit")?;
    stdout.flush()?;
    Ok(())

}


// Draw the filter menu
pub fn draw_filter_menu() -> std::io::Result<()> {

    let mut stdout = stdout();
    let mut process_manager = ProcessManager::new(); // Initialize process manager
    let mut scroll_offset: usize = 0;                // Track scrolling position
    let display_limit: usize = 20;                   // Number of processes visible at a time
    let process_len = process_manager.get_processes().len(); // Total number of processes

    loop{
        process_manager.refresh();
        let processes = process_manager.get_processes().clone();
        if handle_filter_key_event(&mut scroll_offset, display_limit, process_len)? {
            break; // Exit loop if 'q' is pressed
        }

        draw_processes(&processes, scroll_offset, display_limit)?;
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        writeln!(stdout, "1. Sort  |  2. Filter  |  [←] Back")?;

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
    loop{
        process_manager.refresh();
        let processes = process_manager.get_processes().clone();
            if handle_sort_key_event(&mut scroll_offset, display_limit, process_len)? {
                break; // Exit loop if 'q' is pressed
            }
            draw_processes(&processes, scroll_offset, display_limit)?;
            execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
            writeln!(stdout, "1. Sort by PID  |  2. Sort by MEM  |  3. Sort by PPID  |  4. Sort by Start  |  [←] Back")?;


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
    
            // Apply sorting based on sort_mode
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
    
            // Clear and draw header
            execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
            writeln!(
                stdout,
                "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>12} {:>10}",
                "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "USER", "STATUS",
            )?;
    
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
    
                writeln!(
                    stdout,
                    "{:<6} {:<18} {:>6.2} {:>10} {:>8} {:>12} {:>12} {:>10}",
                    process.pid,
                    name,
                    process.cpu_usage,
                    memory_mb,
                    process.parent_pid.unwrap_or(0),
                    process.start_time,
                    user_display,
                    process.status.trim(),
                )?;
            }
    
            execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
            writeln!(stdout, "[↑] Scroll Up  |  [↓] Scroll Down")?;
            execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
            writeln!(stdout, "[←] Back")?;
    
            stdout.flush()?;
            sleep(Duration::from_millis(100));
        }
    
        Ok(())
    }