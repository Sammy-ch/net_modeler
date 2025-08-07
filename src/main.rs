mod components;

use crate::{app::AppModel, components::network::Network};
use std::collections::VecDeque;
mod app;

fn main() {
    let mut model = AppModel::init("Network Visualization");
    let mut message_queue = VecDeque::new();

    while !model.rl.window_should_close() {
        model.handle_input(&mut message_queue);

        while let Some(msg) = message_queue.pop_front() {
            model.update(msg);
        }

        model.init_network_canvas(&mut message_queue);
    }
}
