use std::collections::VecDeque;

use raylib::{imgui::RayImGUITrait, prelude::RaylibDrawHandle};

use crate::app::AppMsg;

pub fn init_ui(rhandle: &RaylibDrawHandle, message_queue: &mut VecDeque<AppMsg>) {
    if let Some(imgui_handle) = rhandle.begin_imgui() {
        if let Some(win) = imgui_handle
            .window("Net Modeler")
            .size([300.0, 100.0], ::imgui::Condition::Always)
            .position([0.0, 0.0], ::imgui::Condition::Always)
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .begin()
        {
            imgui_handle.button("Add Node").then(|| {
                let x = rand::random_range(50..750) as f64;
                let y = rand::random_range(50..750) as f64;
                message_queue.push_back(AppMsg::AddPoint((x, y)));
            });
            win.end();
        }
    }
}
