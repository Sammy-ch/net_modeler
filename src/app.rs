use crate::components::ui::UiState;
use std::collections::VecDeque;

use crate::{
    Network,
    components::{
        network::{Node, load_network_links},
        ui::init_ui,
    },
};
use petgraph::graph::NodeIndex;
use raylib::prelude::*;

pub struct AppModel {
    network: Network,
    pub rl: RaylibHandle,
    pub rthread: RaylibThread,
    dragged_node: Option<(NodeIndex, f64, f64)>,
    canvas_offset_x: i32,
    uistate: UiState,
}

#[derive(Debug)]
pub enum AppMsg {
    AddPoint((f64, f64)),
    StartDrag(NodeIndex, f64, f64),
    UpdateDrag(f64, f64),
    EndDrag,
}

impl AppModel {
    pub fn init(title: impl AsRef<str>) -> AppModel {
        let mut network = Network::new();

        let network_links = load_network_links("configuration/network.csv").unwrap();
        let canvas_offset_x = 250;
        for link in &network_links {
            if !network.node_indices.contains_key(&link.source_node) {
                let source_node = Node {
                    id: link.source_node.clone(),
                    point: (
                        rand::random_range(50..(750 - canvas_offset_x)),
                        rand::random_range(50..550),
                    ),
                };
                network.add_node(source_node);
            }

            if !network.node_indices.contains_key(&link.destination_node) {
                let destination_node = Node {
                    id: link.destination_node.clone(),
                    point: (
                        rand::random_range(50..(750 - canvas_offset_x)),
                        rand::random_range(50..550),
                    ),
                };
                network.add_node(destination_node);
            }
        }

        for link in network_links {
            network.add_link(link).expect("Failed to add link");
        }

        let (rl, rthread) = raylib::init().size(800, 600).title(title.as_ref()).build();

        let width = rl.get_screen_width();
        let height = rl.get_screen_height();

        network.apply_force_directed_layout(width, height, 100, None);

        AppModel {
            network,
            rl,
            rthread,
            dragged_node: None,
            canvas_offset_x,
            uistate: UiState::default(),
        }
    }

    pub fn handle_input(&mut self, message_queue: &mut VecDeque<AppMsg>) {
        let mouse_pos = self.rl.get_mouse_position();
        if self
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            if let Some(node_idx) = self.network.find_node_at_point(
                mouse_pos.x as f64 - self.canvas_offset_x as f64,
                mouse_pos.y as f64,
                18.0,
            ) {
                // Calculate offset from node center to mouse click
                let node = self.network.graph.node_weight(node_idx).unwrap();
                let offset_x =
                    mouse_pos.x as f64 - node.point.0 as f64 - self.canvas_offset_x as f64;
                let offset_y = mouse_pos.y as f64 - node.point.1 as f64;
                message_queue.push_back(AppMsg::StartDrag(node_idx, offset_x, offset_y));
            }
        }
        match (
            self.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
            self.rl
                .is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
            self.dragged_node.is_some(),
        ) {
            (true, _, true) => {
                message_queue.push_back(AppMsg::UpdateDrag(
                    mouse_pos.x as f64 - self.canvas_offset_x as f64,
                    mouse_pos.y as f64,
                ));
            }
            (_, true, true) => {
                message_queue.push_back(AppMsg::EndDrag);
            }
            _ => {}
        }
    }

    pub fn update(&mut self, msg: AppMsg) {
        match msg {
            AppMsg::AddPoint((x, y)) => {
                let node = Node {
                    id: format!("node{}", self.network.graph.node_count()),
                    point: (x as i32, y as i32),
                };
                self.network.add_node(node);
            }
            AppMsg::StartDrag(node_idx, offset_x, offset_y) => {
                self.dragged_node = Some((node_idx, offset_x, offset_y));
            }
            AppMsg::UpdateDrag(mouse_x, mouse_y) => {
                if let Some((node_idx, offset_x, offset_y)) = self.dragged_node {
                    if let Some(node) = self.network.graph.node_weight_mut(node_idx) {
                        // Set node position to mouse position minus offset
                        node.point.0 = (mouse_x - offset_x) as i32;
                        node.point.1 = (mouse_y - offset_y) as i32;
                        // Clamp to screen bounds
                        node.point.0 = node.point.0.clamp(50, self.rl.get_screen_width() - 50);
                        node.point.1 = node.point.1.clamp(50, self.rl.get_screen_height() - 50);
                    }
                }
            }
            AppMsg::EndDrag => {
                self.dragged_node = None;
            }
        }
    }

    pub fn init_network_canvas(&mut self, message_queue: &mut VecDeque<AppMsg>) {
        self.rl.draw(&self.rthread, |mut rhandle| {
            rhandle.clear_background(Color::BLACK);

            let canvas_width = rhandle.get_screen_width() - self.canvas_offset_x;
            let canvas_height = rhandle.get_screen_height();

            rhandle.draw_scissor_mode(
                self.canvas_offset_x,
                0,
                canvas_width,
                canvas_height,
                |mut handle| {
                    for (link, src_node, dest_node) in self.network.links() {
                        let start_pos = Vector2 {
                            x: src_node.point.0 as f32 + self.canvas_offset_x as f32,
                            y: src_node.point.1 as f32,
                        };
                        let end_pos = Vector2 {
                            x: dest_node.point.0 as f32 + self.canvas_offset_x as f32,
                            y: dest_node.point.1 as f32,
                        };

                        let mid_x =
                            (src_node.point.0 + dest_node.point.0) / 2 + self.canvas_offset_x;
                        let mid_y = (src_node.point.1 + dest_node.point.1) / 2;

                        let offset = if link.link_id.as_bytes()[0] % 2 == 0 {
                            30.0
                        } else {
                            -30.0
                        };

                        handle.draw_line_bezier(start_pos, end_pos, 2.0, Color::WHEAT);

                        let weight_text = link.weight.to_string();
                        let font_size = 18;
                        let text_width = handle.measure_text(weight_text.as_str(), font_size);
                        let text_height = font_size;

                        let text_x = mid_x - text_width / 2;
                        let text_y = (mid_y as f32 + offset - text_height as f32 / 2.0) as i32;

                        handle.draw_text(
                            weight_text.as_str(),
                            text_x,
                            text_y,
                            font_size,
                            Color::RAYWHITE,
                        );
                    }

                    // Draw nodes
                    for node in self.network.nodes() {
                        handle.draw_circle(
                            node.point.0 + self.canvas_offset_x,
                            node.point.1,
                            18.0,
                            Color::WHITE,
                        );

                        let text = node.id.as_str();
                        let font_size = 12;
                        let text_width = handle.measure_text(text, font_size);
                        let text_height = font_size;

                        let text_x = node.point.0 + self.canvas_offset_x - text_width / 2;
                        let text_y = node.point.1 - text_height / 2;

                        handle.draw_text(text, text_x, text_y, font_size, Color::BLACK);
                    }

                    // init imgui
                    init_ui(&handle, message_queue, &self.network, &mut self.uistate);
                },
            );
        });
    }
}
