mod components;
use crate::components::network::{Network, Node, load_network_links};
use petgraph::graph::NodeIndex;
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::RelmApp;
use relm4::SimpleComponent;
use relm4::gtk;
use relm4::gtk::cairo;
use relm4::gtk::prelude::{
    BoxExt, DrawingAreaExtManual, GestureSingleExt, GtkWindowExt, WidgetExt,
};
use std::error::Error;

struct AppModel {
    network: Network,
    dragged_node: Option<NodeIndex>,
    needs_redraw: bool,
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

struct AppWidgets {
    drawing_area: gtk::DrawingArea,
}

impl SimpleComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type Init = Network;
    type Widgets = AppWidgets;
    type Root = gtk::Window;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("Net Modeler")
            .default_width(800)
            .default_height(600)
            .build()
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        widgets.drawing_area.queue_draw();
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::DrawRequested => self.needs_redraw = true,
            AppMsg::AddPoint((x, y)) => {
                let new_node = components::network::Node {
                    id: format!("Node{}", self.network.graph.node_count() + 1),
                    point: (x, y),
                };
                self.network.add_node(new_node);
            }
            AppMsg::UpdateLayout => {
                self.network
                    .apply_force_directed_layout(800.0, 600.0, 1, self.dragged_node);
                self.needs_redraw = true;
            }
            AppMsg::StartDrag(node_idx, x, y) => {
                self.dragged_node = Some(node_idx);
                let node = self.network.graph.node_weight_mut(node_idx).unwrap();
                node.point = (x, y);

                self.needs_redraw = true;
            }
            AppMsg::UpdateDrag(offset_x, offset_y) => {
                if let Some(node_idx) = self.dragged_node {
                    let node = self.network.graph.node_weight_mut(node_idx).unwrap();
                    let (start_x, start_y) = node.point;
                    node.point = (
                        (start_x + offset_x).clamp(50.0, 800.0 - 50.0),
                        (start_y + offset_y).clamp(50.0, 600.0 - 50.0),
                    );
                }

                self.needs_redraw = true;
            }

            AppMsg::EndDrag => {
                self.dragged_node = None;

                self.needs_redraw = true;
            }
        }
    }

    fn init(
        network: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let drawing_area = gtk::DrawingArea::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let network_clone = network.clone();
        drawing_area.set_draw_func(move |_drawing_area, cr, _width, _height| {
            cr.set_source_rgb(52.0 / 255.0, 52.0 / 255.0, 52.0 / 255.0);
            cr.paint().expect("Failed to paint background");

            for (link, src_node, dest_node) in network_clone.links() {
                cr.set_source_rgb(0.5, 0.5, 0.5);
                cr.set_line_width(2.0);

                // Quadratic bezier curve for edges
                let mid_x = (src_node.point.0 + dest_node.point.0) / 2.0;
                let mid_y = (src_node.point.1 + dest_node.point.1) / 2.0;

                // Offset control point based on link_id to alternate curve direction
                let offset = if link.link_id.as_bytes()[0] % 2 == 0 {
                    30.0
                } else {
                    -30.0
                };
                let control_x = mid_x;
                let control_y = mid_y + offset;

                cr.move_to(src_node.point.0, src_node.point.1);
                cr.curve_to(
                    control_x,
                    control_y,
                    control_x,
                    control_y,
                    dest_node.point.0,
                    dest_node.point.1,
                );
                cr.stroke().expect("Failed to draw links");

                cr.move_to(mid_x + 5.0, mid_y - 10.0);
                cr.set_source_rgb(223.0 / 255.0, 223.0 / 255.0, 223.0 / 255.0);
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                cr.set_font_size(12.0);
                cr.show_text(&format!("{}", link.weight))
                    .expect("Failed to draw link text");
            }

            for node in network_clone.nodes() {
                cr.set_source_rgb(226.0 / 255.0, 76.0 / 255.0, 27.0 / 255.0);
                cr.arc(
                    node.point.0,
                    node.point.1,
                    20.0,
                    0.0,
                    2.0 * std::f64::consts::PI,
                );
                cr.fill().expect("Failed to draw node circle");

                // Draw node ID text
                cr.move_to(node.point.0, node.point.1);

                cr.set_source_rgb(223.0 / 255.0, 223.0 / 255.0, 223.0 / 255.0);

                // White text
                cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                cr.set_font_size(16.0);

                // Center the text on the node
                let extents = cr
                    .text_extents(&node.id)
                    .expect("Failed to get text extents");
                cr.move_to(
                    node.point.0 - extents.width() / 2.0 - extents.x_bearing(),
                    node.point.1 - extents.height() / 2.0 - extents.y_bearing(),
                );
                cr.show_text(&node.id).expect("Failed to draw node text");
            }
        });

        let click_controller = gtk::GestureClick::new();
        click_controller.set_button(1);

        click_controller.connect_pressed({
            let sender = sender.clone();
            let network = network.clone();
            move |_, _npress, x, y| {
                if network.find_node_at_point(x, y, 20.0).is_none() {
                    sender.input(AppMsg::AddPoint((x, y)));
                }
            }
        });

        drawing_area.add_controller(click_controller);

        // drawing_area.add_controller(&click_controller);

        // let drag_controller = gtk::GestureDrag::new();
        // drag_controller.set_button(1);
        //
        // drag_controller.connect_drag_begin({
        //     let sender = sender.clone();
        //     let network = network.clone();
        //     move |gesture, x, y| {
        //         if let Some(node_idx) = network.find_node_at_point(x, y, 20.0) {
        //             sender.input(AppMsg::StartDrag(node_idx, x, y));
        //             gesture.set_state(gtk::EventSequenceState::Claimed);
        //         }
        //     }
        // });
        //
        // drag_controller.connect_drag_update({
        //     let sender = sender.clone();
        //     move |_, offset_x, offset_y| {
        //         sender.input(AppMsg::UpdateDrag(offset_x, offset_y));
        //     }
        // });
        //
        // drag_controller.connect_drag_end({
        //     let sender = sender.clone();
        //     move |_, _, _| {
        //         sender.input(AppMsg::EndDrag);
        //     }
        // });
        //
        // drawing_area.add_controller(drag_controller);
        //
        let vbox = gtk::Box::builder().spacing(5).build();

        root.set_child(Some(&vbox));
        vbox.append(&drawing_area);

        let widgets = AppWidgets { drawing_area };

        let model = AppModel {
            network,
            dragged_node: None,
            needs_redraw: false,
        };

        ComponentParts { model, widgets }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let network_links = load_network_links()?;
    let mut network = Network::new();

    for link in &network_links {
        if !network.node_indices.contains_key(&link.source_node) {
            let source_node = Node {
                id: link.source_node.clone(),
                point: (
                    rand::random_range(50.0..750.0),
                    rand::random_range(50.0..550.0),
                ),
            };

            network.add_node(source_node);
        }

        if !network.node_indices.contains_key(&link.destination_node) {
            let destination_node = Node {
                id: link.destination_node.clone(),
                point: (
                    rand::random_range(50.0..750.0),
                    rand::random_range(50.0..550.0),
                ),
            };

            network.add_node(destination_node);
        }
    }

    for link in network_links {
        network.add_link(link).expect("Failed to add link");
    }

    network.apply_force_directed_layout(800.0, 600.0, 50, None);

    let app = RelmApp::new("com.test.net_modeler");
    app.run::<AppModel>(network);

    Ok(())
}
