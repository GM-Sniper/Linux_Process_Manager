// mod process;

// use crossterm::{
//     cursor,
//     event::{self, Event, KeyCode},
//     execute,
//     terminal::{self, ClearType},
// };
// use std::{
//     io::{stdout, Write},
//     time::Duration,
// };
// use process::ProcessManager;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // Set up terminal
//     terminal::enable_raw_mode()?;
//     let mut stdout = stdout();
//     execute!(stdout, terminal::EnterAlternateScreen)?;

//     // Create process manager
//     let mut process_manager = ProcessManager::new();
    
//     // Main loop
//     loop {
//         // Check for exit key
//         if event::poll(Duration::from_millis(100))? {
//             if let Event::Key(key_event) = event::read()? {
//                 if key_event.code == KeyCode::Char('q') {
//                     break;
//                 }
//             }
//         }
        
//         // Clear screen
//         execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        
//         // Refresh process data
//         process_manager.refresh();
        
//         // Get processes
//         let processes = process_manager.get_processes();

//         // Print header without tabs
//         println!("PID    NAME           CPU  MEM(KB)");
        
//         // Print each process with explicit fixed-width formatting
//         for process in processes.iter().take(20) {
//             let name = if process.name.len() > 12 {
//                 format!("{}...", &process.name[0..9])
//             } else {
//                 format!("{:<12}", process.name)
//             };
            
//             let memory_kb = process.memory_usage / 1024;
            
//             // Using explicit formatting with specific character counts
//             println!("{:<6} {:<14} {:<4.1} {}", 
//                 process.pid, name, process.cpu_usage, memory_kb);
//         }
        
//         // Footer
//         println!("q:quit");
        
//         // Refresh every second
//         std::thread::sleep(Duration::from_secs(1));
//     }
    
//     // Clean up terminal
//     execute!(stdout, terminal::LeaveAlternateScreen)?;
//     terminal::disable_raw_mode()?;
    
//     Ok(())
// }
