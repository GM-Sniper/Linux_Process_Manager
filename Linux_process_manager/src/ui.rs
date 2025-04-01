
use sysinfo::{ProcessExt, System, SystemExt, PidExt, UserExt, Process};
use crossterm::{
    cursor, execute, terminal,
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
pub fn draw_processes(processes: &[Process], scroll_offset: usize, display_limit: usize) -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, cursor::MoveTo(0, 0))?;

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
