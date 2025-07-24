mod components;
use crate::components::network::{Network, Node, load_network_links};
use log::info;
use petgraph::dot::{Config, Dot};
use rand::Rng;
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::RelmApp;
use relm4::RelmWidgetExt;
use relm4::SimpleComponent;
use relm4::gtk;
use relm4::gtk::prelude::BoxExt;
use relm4::gtk::prelude::ButtonExt;
use relm4::gtk::prelude::GtkWindowExt;
use relm4::gtk::prelude::OrientableExt;
use std::error::Error;

struct AppModel {
    network: Network,
}

#[derive(Debug)]
enum AppMsg {
    DrawRequested,
    AddPoint((f64, f64)),
}

struct AppWidgets {
    canvas: gtk::DrawingArea,
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Input = AppMsg;
    type Output = ();
    type Init = Network;

    view! {
           gtk::Window{
               set_title: Some("Net Modeler"),
               set_default_width: 300,
               set_default_height: 100,

                   }
    }

    fn init(
        network: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel { network };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let network_links = load_network_links().unwrap();
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

    let app = RelmApp::new("com.test.net_modeler");
    app.run::<AppModel>(network);

    Ok(())
}
