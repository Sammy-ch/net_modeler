use petgraph::{
    graph::{NodeIndex, UnGraph},
    visit::EdgeRef,
};

use serde::Deserialize;
use std::{collections::HashMap, error::Error, fmt::Display};

#[derive(Debug)]
pub enum NetworkError {
    NodeNotFound(String),
    Io(std::io::Error),
    Csv(csv::Error),
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::NodeNotFound(id) => write!(f, "Node not found: {}", id),
            NetworkError::Io(err) => write!(f, "IO error: {}", err),
            NetworkError::Csv(err) => write!(f, "CSV error: {}", err),
        }
    }
}

impl Error for NetworkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NetworkError::Io(err) => Some(err),
            NetworkError::Csv(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NetworkError {
    fn from(value: std::io::Error) -> Self {
        NetworkError::Io(value)
    }
}

impl From<csv::Error> for NetworkError {
    fn from(value: csv::Error) -> Self {
        NetworkError::Csv(value)
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Node {
    pub id: String,
    pub point: (f64, f64),
}

#[derive(Debug, Deserialize, Clone)]
pub struct Link {
    pub link_id: String,
    pub source_node: String,
    pub destination_node: String,
    pub capacity: u8,
    pub weight: u8,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub graph: UnGraph<Node, Link>,
    pub node_indices: HashMap<String, NodeIndex>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            graph: UnGraph::default(),
            node_indices: HashMap::new(),
        }
    }

    pub fn find_node_at_point(&self, x: f64, y: f64, radius: f64) -> Option<NodeIndex> {
        for (i, node) in self.nodes().enumerate() {
            let dx = x - node.point.0;
            let dy = y - node.point.1;
            if (dx * dx + dy * dy).sqrt() <= radius {
                return Some(NodeIndex::new(i));
            }
        }
        None
    }
    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        if let Some(&index) = self.node_indices.get(&node.id) {
            index
        } else {
            let index = self.graph.add_node(node.clone());
            self.node_indices.insert(node.id, index);
            index
        }
    }

    pub fn add_link(&mut self, link: Link) -> Result<(), NetworkError> {
        let source_index = *self
            .node_indices
            .get(&link.source_node)
            .ok_or_else(|| NetworkError::NodeNotFound(link.source_node.clone()))?;

        let destination_source = *self
            .node_indices
            .get(&link.destination_node)
            .ok_or_else(|| NetworkError::NodeNotFound(link.destination_node.clone()))?;

        self.graph.add_edge(source_index, destination_source, link);
        Ok(())
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.graph.node_weights()
    }

    pub fn links(&self) -> impl Iterator<Item = (Link, &Node, &Node)> {
        self.graph.edge_references().map(|edge_ref| {
            let (source_idx, dest_idx) = self.graph.edge_endpoints(edge_ref.id()).unwrap();
            let source_node = self.graph.node_weight(source_idx).unwrap();
            let dest_node = self.graph.node_weight(dest_idx).unwrap();
            (edge_ref.weight().clone(), source_node, dest_node)
        })
    }
    pub fn apply_force_directed_layout(
        &mut self,
        width: f64,
        height: f64,
        iterations: usize,
        pinned_node: Option<NodeIndex>,
    ) -> f64 {
        let k = (width * height / self.graph.node_count().max(1) as f64).sqrt() * 1.5;
        let cooling_factor = 0.9;
        let mut temp = width / 5.0;
        let mut max_displacement: f64 = 0.0;

        let mut displacements: Vec<(f64, f64)> = vec![(0.0, 0.0); self.graph.node_count()];

        for _ in 0..iterations {
            displacements.iter_mut().for_each(|d| *d = (0.0, 0.0));

            // Repulsive forces
            for i in 0..self.graph.node_count() {
                if pinned_node == Some(NodeIndex::new(i)) {
                    continue; // Skip pinned node
                }
                for j in i + 1..self.graph.node_count() {
                    let node_i = self.graph.node_weight(NodeIndex::new(i)).unwrap();
                    let node_j = self.graph.node_weight(NodeIndex::new(j)).unwrap();
                    let dx = node_j.point.0 - node_i.point.0;
                    let dy = node_j.point.1 - node_i.point.1;
                    let dist = (dx * dx + dy * dy).sqrt().max(1e-2);
                    let force = k * k / dist;
                    let fx = force * dx / dist;
                    let fy = force * dy / dist;
                    displacements[i].0 -= fx;
                    displacements[i].1 -= fy;
                    displacements[j].0 += fx;
                    displacements[j].1 += fy;
                }
            }

            // Attractive forces
            for edge_ref in self.graph.edge_references() {
                let (src_idx, dest_idx) = self.graph.edge_endpoints(edge_ref.id()).unwrap();
                if pinned_node == Some(src_idx) || pinned_node == Some(dest_idx) {
                    continue; // Skip edges involving pinned node
                }
                let src_node = self.graph.node_weight(src_idx).unwrap();
                let dest_node = self.graph.node_weight(dest_idx).unwrap();
                let dx = dest_node.point.0 - src_node.point.0;
                let dy = dest_node.point.1 - src_node.point.1;
                let dist = (dx * dx + dy * dy).sqrt().max(1e-2);
                let force = dist * dist / k;
                let fx = force * dx / dist;
                let fy = force * dy / dist;
                displacements[src_idx.index()].0 += fx;
                displacements[src_idx.index()].1 += fy;
                displacements[dest_idx.index()].0 -= fx;
                displacements[dest_idx.index()].1 -= fy;
            }

            // Update positions
            max_displacement = 0.0;
            for i in 0..self.graph.node_count() {
                if pinned_node == Some(NodeIndex::new(i)) {
                    continue; // Skip pinned node
                }
                let disp = displacements[i];
                let disp_len = (disp.0 * disp.0 + disp.1 * disp.1).sqrt().max(1e-2);
                max_displacement = max_displacement.max(disp_len);
                let factor = temp / disp_len;
                let node = self.graph.node_weight_mut(NodeIndex::new(i)).unwrap();
                node.point.0 += disp.0 * factor;
                node.point.1 += disp.1 * factor;

                node.point.0 = node.point.0.clamp(50.0, width - 50.0);
                node.point.1 = node.point.1.clamp(50.0, height - 50.0);
            }

            temp *= cooling_factor;
        }
        max_displacement
    }
}

pub fn load_network_links() -> Result<Vec<Link>, NetworkError> {
    let mut rdr = csv::Reader::from_path("configuration/network.csv")?;
    let mut network_links: Vec<Link> = Vec::new();

    for network in rdr.deserialize() {
        let loaded_link: Link = network?;
        network_links.push(loaded_link);
    }

    Ok(network_links)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn network_load_successful() {
        let csv_content = "link_id,source_node,destination_node,capacity,weight\n\
                           link_A,Node1,Node2,100,10\n\
                           link_B,Node2,Node3,50,5\n\
                           link_C,Node1,Node3,75,8\n";

        let path = "test_configuration/test-network.csv";
        std::fs::create_dir_all("test_configuration")
            .expect("Failed to create test configuration directory");
        std::fs::write(path, csv_content).expect("Failed to write dummy CSV");

        let mut rdr = csv::Reader::from_path(path).unwrap();
        let mut network_links: Vec<Link> = Vec::new();

        for network in rdr.deserialize() {
            let loaded_link: Link = network.unwrap();
            network_links.push(loaded_link);
        }

        let mut network = Network::new();

        // First, add nodes
        for link in &network_links {
            network.add_node(Node {
                id: link.source_node.clone(),
                point: (0.0, 0.0),
            });
            network.add_node(Node {
                id: link.destination_node.clone(),

                point: (0.0, 0.0),
            });
        }

        // Then, add links
        for link in network_links {
            network.add_link(link).expect("Failed to add link");
        }

        assert_eq!(network.graph.node_count(), 3); // Node1, Node2, Node3
        assert_eq!(network.graph.edge_count(), 3); // link_A, link_B, link_C

        // Verify specific nodes and edges exist
        let node1_idx = network.node_indices.get("Node1").expect("Node1 not found");
        let node2_idx = network.node_indices.get("Node2").expect("Node2 not found");
        let node3_idx = network.node_indices.get("Node3").expect("Node3 not found");

        assert_eq!(network.graph[*node1_idx].id, "Node1");
        assert_eq!(network.graph[*node2_idx].id, "Node2");
        assert_eq!(network.graph[*node3_idx].id, "Node3");

        // Check if an edge exists from Node1 to Node2
        let edge_ref = network.graph.find_edge(*node1_idx, *node2_idx);
        assert!(edge_ref.is_some());
        let edge_weight = network.graph.edge_weight(edge_ref.unwrap()).unwrap();
        assert_eq!(edge_weight.capacity, 100);
        assert_eq!(edge_weight.weight, 10);

        std::fs::remove_file(path).expect("Failed to remove dummy CSV");
        std::fs::remove_dir("test_configuration").expect("Failed to remove Test Dir");
    }

    #[test]
    fn test_node_addition_deduplication() {
        let mut network = Network::new();
        let node1 = Node {
            id: "A".to_string(),
            point: (0.0, 0.0),
        };
        let node2 = Node {
            id: "B".to_string(),
            point: (0.0, 0.0),
        };

        let idx_a1 = network.add_node(node1.clone());
        let idx_b = network.add_node(node2.clone());
        let idx_a2 = network.add_node(node1.clone()); // Add the same node again

        assert_eq!(network.graph.node_count(), 2);
        assert_eq!(network.node_indices.len(), 2);
        assert_eq!(idx_a1, idx_a2); // Should return the same index for the same node ID
        assert_ne!(idx_a1, idx_b);
    }

    #[test]
    fn test_link_addition_error_handling() {
        let mut network = Network::new();
        let node_a = Node {
            id: "A".to_string(),
            point: (0.0, 0.0),
        };
        network.add_node(node_a);

        // Try to add a link with a non-existent destination node
        let invalid_link = Link {
            link_id: "link_invalid".to_string(),
            source_node: "A".to_string(),
            destination_node: "NonExistent".to_string(),
            capacity: 10,
            weight: 1,
        };
        let result = network.add_link(invalid_link);
        assert!(result.is_err());
        match result.unwrap_err() {
            NetworkError::NodeNotFound(id) => assert_eq!(id, "NonExistent"),
            _ => panic!("Expected NodeNotFound error"),
        }
    }
}
