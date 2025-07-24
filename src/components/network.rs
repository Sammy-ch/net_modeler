use petgraph::graph::{NodeIndex, UnGraph};

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

#[derive(Debug, Deserialize)]
pub struct Link {
    pub link_id: String,
    pub source_node: String,
    pub destination_node: String,
    pub capacity: u8,
    pub weight: u8,
}

#[derive(Debug)]
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

    // fn nodes(&self) -> impl Iterator<Item = &Node> {
    //     self.graph.node_weights()
    // }
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
