// Project: Linux Process Manager
mod process; 
// mod ui;
use process::ProcessManager;

// This is a simple Linux process manager that displays system processes in a terminal interface (like ps).
// fn main() {
//     let mut process_manager = ProcessManager::new();
//     process_manager.refresh();
//     process_manager.print_processes();
// }

// Adding the refersh ability
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
}; // crossterm crate for terminal manipulation
use std::{io::{stdout, Write}, thread, time::Duration}; // std crate for IO and threading

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal
    terminal::enable_raw_mode()?; 
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?; 

    let mut process_manager = ProcessManager::new();
    
    let mut scroll_offset: usize = 0;  // Track scrolling position
    let display_limit: usize = 20;     // Number of processes visible at a time

    loop {
        // Handle key press events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => break, // Quit
                    KeyCode::Up => {
                        if scroll_offset > 0 {
                            scroll_offset -= 1; // Scroll up
                        }
                    }
                    KeyCode::Down => {
                        if scroll_offset < process_manager.get_processes().len().saturating_sub(display_limit) {
                            scroll_offset += 1; // Scroll down
                        }
                    }
                    _ => {}
                }
            }
        }

        // Move cursor to the top without clearing screen
        execute!(stdout, cursor::MoveTo(0, 0))?;

        // Refresh process data
        process_manager.refresh();
        let processes = process_manager.get_processes();

        writeln!(
            stdout,
            "{:<6} {:<18} {:>6} {:>10} {:>8} {:>12} {:>12} {:>10}",
            "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "USER", "STATUS",
        )?;

        // Calculate range of processes to display based on scroll_offset
        let start_index = scroll_offset;
        let end_index = (scroll_offset + display_limit).min(processes.len());

        for (i, process) in processes.iter().enumerate().take(end_index).skip(start_index) {
            execute!(stdout, cursor::MoveTo(0, (i - start_index + 1) as u16))?; 

            let name = if process.name.len() > 15 {
                format!("{:.12}...", process.name)
            } else {
                process.name.clone()
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
                process.user.clone().unwrap_or_default(),
                process.status,
            )?;
        }

        // Print scroll instructions
        execute!(stdout, cursor::MoveTo(0, (display_limit + 2) as u16))?;
        writeln!(stdout, "[↑] Scroll Up  |  [↓] Scroll Down \n")?;
        execute!(stdout, cursor::MoveTo(0, (display_limit + 3) as u16))?;
        writeln!(stdout, "1. Filter  |  2. Change Priority  |  3. Kill/Stop Process |  [Q] Quit")?;

        stdout.flush()?;
        thread::sleep(Duration::from_millis(100));
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

