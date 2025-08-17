use std::collections::VecDeque;

use raylib::{imgui::RayImGUITrait, prelude::RaylibDrawHandle};

use crate::{app::AppMsg, components::network::Network};

#[derive(Debug, Default)]
pub struct UiState {
    selected_start_index: usize,
    selected_end_index: usize,
}

pub fn init_ui(
    rhandle: &RaylibDrawHandle,
    message_queue: &mut VecDeque<AppMsg>,
    network: &Network,
    ui_state: &mut UiState,
) {
    if let Some(ui) = rhandle.begin_imgui() {
        if let Some(win) = ui
            .window("Net Modeler")
            .size([250.0, 600.0], ::imgui::Condition::Always)
            .position([0.0, 0.0], ::imgui::Condition::Always)
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .begin()
        {
            ui.text("Network Tool");
            ui.separator();

            ui.button("Add Node").then(|| {
                let x = rand::random_range(50..750) as f64 + 200.0;
                let y = rand::random_range(50..750) as f64;
                message_queue.push_back(AppMsg::AddPoint((x, y)));
            });

            ui.separator();
            ui.text("Shortest path");
            let mut node_ids: Vec<String> = network.node_indices.keys().cloned().collect();
            node_ids.sort();

            ui.combo(
                "select start node",
                &mut ui_state.selected_start_index,
                &node_ids,
                |node| std::borrow::Cow::Borrowed(node.as_str()),
            );

            ui.combo(
                "select end node",
                &mut ui_state.selected_end_index,
                &node_ids,
                |node| std::borrow::Cow::Borrowed(node.as_str()),
            );

            win.end();
        }
    }
}
