// Project: Linux Process Manager
mod process; 
mod ui;
use process::ProcessManager;

// This is a simple Linux process manager that displays system processes in a terminal interface (like ps).
// fn main() {
//     let mut process_manager = ProcessManager::new();
//     process_manager.refresh();
//     process_manager.print_processes();
// }

// Adding the refersh ability
use crossterm::{
    event::{self, Event, KeyCode},
}; // crossterm crate for terminal manipulation
use std::{thread, time::Duration}; // std crate for IO and threading

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ui::setup_terminal()?; // Setup terminal for raw mode and alternate screen

    let mut process_manager = ProcessManager::new(); // Initialize process manager
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
        // Refresh process data
        process_manager.refresh();
        let processes = process_manager.get_processes();

        ui::draw_processes(&processes, scroll_offset, display_limit)?; // Print processes
        ui::draw_menu(display_limit)?; // Print menu
        thread::sleep(Duration::from_millis(100));
    }

    ui::restore_terminal()?; // Restore terminal to normal mode

    Ok(())
}

