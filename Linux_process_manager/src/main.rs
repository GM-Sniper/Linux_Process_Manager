// Project: Linux Process Manager
mod process; 

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

fn main() -> Result<(), Box<dyn std::error::Error>> { // Main function with error handling
    // Set up terminal
    terminal::enable_raw_mode()?; // Raw mode for better control
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?; // Use alternate screen for better UI

    // Create a new ProcessManager instance
    // This will be used to manage and display processes
    let mut process_manager = ProcessManager::new();

    // This loop will refresh the process data and display it in the terminal
    // The loop will run indefinitely until the user presses 'q'
    loop {
        // Logic for q (quit) key press
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Move cursor of screen to the top (avoids clearing screen)
        execute!(stdout, cursor::MoveTo(0, 0))?;

        // Refresh process data
        process_manager.refresh();
        let processes = process_manager.get_processes();
        // Print the header (once)
        writeln!(
            stdout,
            "{:<6} {:<16} {:>6} {:>10} {:>8} {:>12} {:>8} {:>10}",
            "PID", "NAME", "CPU%", "MEM(MB)", "PPID", "START", "USER", "STATUS",
        )?;

        // Print each process in a fixed row position (taking only the first n processes)
        let mut n = 20; // Number of processes to display
        if processes.len() < n {
            n = processes.len(); // Adjust n if there are fewer processes
        }
        for (i, process) in processes.iter().enumerate().take(n) {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?; // Move to line i+1

            let name = if process.name.len() > 15 {
                format!("{:.12}...", process.name)
            } else {
                process.name.clone()
            };

            let memory_mb = process.memory_usage / (1024 * 1024); // Convert memory to MB

            // Print process information (added STATUS at the end)
            writeln!(
                stdout,
                "{:<6} {:<16} {:>6.2} {:>10} {:>8} {:>12} {:>8} {:>10}",
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



        // Flush output to update the screen
        stdout.flush()?;

        // Wait before the next refresh
        thread::sleep(Duration::from_secs(1));
    }

    // Restore terminal to normal
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

