use raylib::{imgui::RayImGUITrait, prelude::RaylibDrawHandle};

pub fn init_ui(rhandle: &RaylibDrawHandle) {
    rhandle.draw_imgui(|ui| {
        ui.show_demo_window(&mut true);
    });
}

