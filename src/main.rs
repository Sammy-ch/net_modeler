use serde::Deserialize;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
enum NetworkError {
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct Node {
    id: String,
    point: (u8, u8),
}

#[derive(Debug, Deserialize)]
struct Link {
    link_id: String,
    source_node: String,
    destination_node: String,
    capacity: u8,
    weight: u8,
}

fn main() {
    todo!();
}
