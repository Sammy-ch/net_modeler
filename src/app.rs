use crate::{
    Network,
    components::network::{Node, load_network_links},
};
use petgraph::graph::NodeIndex;
use raylib::prelude::*;

pub struct AppModel {
    network: Network,
    pub rl: RaylibHandle,
    pub rthread: RaylibThread,
}

#[derive(Debug)]
enum AppMsg {
    DrawRequested,
    AddPoint((f64, f64)),
    UpdateLayout,
    StartDrag(NodeIndex, f64, f64),
    UpdateDrag(f64, f64),
    EndDrag,
}
impl AppModel {
    pub fn init(title: impl AsRef<str>) -> AppModel {
        let network_links = load_network_links().unwrap();
        let mut network = Network::new();

        for link in &network_links {
            if !network.node_indices.contains_key(&link.source_node) {
                let source_node = Node {
                    id: link.source_node.clone(),
                    point: (rand::random_range(50..750), rand::random_range(50..550)),
                };

                network.add_node(source_node);
            }

            if !network.node_indices.contains_key(&link.destination_node) {
                let destination_node = Node {
                    id: link.destination_node.clone(),
                    point: (rand::random_range(50..750), rand::random_range(50..550)),
                };

                network.add_node(destination_node);
            }
        }

        for link in network_links {
            network.add_link(link).expect("Failed to add link");
        }

        let (rl, rthread) = raylib::init()
            .width(800)
            .height(600)
            .title(title.as_ref())
            .build();

        let width = rl.get_screen_width();
        let height = rl.get_screen_height();

        network.apply_force_directed_layout(width, height, 100, None);

        AppModel {
            network,
            rl,
            rthread,
        }
    }

    pub fn init_network_canvas(&mut self) {
        self.rl.draw(&self.rthread, |mut d_handle| {
            d_handle.clear_background(Color::BLACK);

            // Draw links
            for (link, src_node, dest_node) in self.network.links() {
                let start_pos = Vector2 {
                    x: src_node.point.0 as f32,
                    y: src_node.point.1 as f32,
                };
                let end_pos = Vector2 {
                    x: dest_node.point.0 as f32,
                    y: dest_node.point.1 as f32,
                };

                let mid_x = (src_node.point.0 + dest_node.point.0) / 2;
                let mid_y = (src_node.point.1 + dest_node.point.1) / 2;

                // Offset control point based on link_id to alternate curve direction
                let offset = if link.link_id.as_bytes()[0] % 2 == 0 {
                    30.0
                } else {
                    -30.0
                };

                // Draw Bezier curve
                d_handle.draw_line_bezier(
                    start_pos,
                    end_pos,
                    2.0, // Thickness
                    Color::WHEAT,
                );

                // Convert link.capacity (u8) to &str and center the text
                let capacity_text = link.capacity.to_string();
                let font_size = 18;
                let text_width = d_handle.measure_text(capacity_text.as_str(), font_size);
                let text_height = font_size; // Approximate height for default font

                // Calculate centered text position with offset
                let text_x = mid_x - text_width / 2;
                let text_y = (mid_y as f32 + offset - text_height as f32 / 2.0) as i32;

                d_handle.draw_text(
                    capacity_text.as_str(),
                    text_x,
                    text_y,
                    font_size,
                    Color::RAYWHITE,
                );
            }

            for node in self.network.nodes() {
                d_handle.draw_circle(node.point.0, node.point.1, 18.0, Color::WHEAT);

                // Measure text width and estimate height for centering
                let text = node.id.as_str();
                let font_size = 12;
                let text_width = d_handle.measure_text(text, font_size);
                let text_height = font_size; // Approximate height for default font

                // Calculate centered position
                let text_x = node.point.0 - text_width / 2;
                let text_y = node.point.1 - text_height / 2;

                d_handle.draw_text(text, text_x, text_y, font_size, Color::BLACK);
            }
        });
    }
}
