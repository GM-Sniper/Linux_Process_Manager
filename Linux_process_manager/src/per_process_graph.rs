//! Per-process graphing module
// This module will provide a UI tab to display CPU and memory usage graphs for a selected process over time.

use ratatui::{Frame, layout::Rect};
use crate::process::ProcessManager;

/// Render the per-process graph tab.
pub fn render_per_process_graph_tab(frame: &mut Frame, area: Rect, process_manager: &ProcessManager) {
    // TODO: Implement process selection and graphing of CPU/memory usage over time.
    // This will likely use a LineChart or similar widget from ratatui.
} 