// Project: Linux Process Manager
mod process;
mod ui;
mod graph;
mod process_log;
mod scripting_rules;
//main to start the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    ui::ui_renderer()
}

