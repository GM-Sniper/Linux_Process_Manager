
use crate::process::ProcessInfo; // ProcessInfo is defined in process.rs
use crossterm::{
    cursor, execute, terminal, terminal::ClearType,
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



        // Convert memory usage from bytes to megabytes
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
