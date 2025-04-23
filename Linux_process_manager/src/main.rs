// Project: Linux Process Manager
mod process;
mod ui;
mod graph;
//main to start the application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    ui::ui_renderer()
}

