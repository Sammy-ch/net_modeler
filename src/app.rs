use crate::Network;
use petgraph::graph::NodeIndex;

struct AppModel {
    network: Network,
    dragged_node: Option<NodeIndex>,
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
