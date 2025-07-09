use std::error::Error;

#[derive(Debug)]
struct Node {
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct Link {
    link_id: String,
    source_node: Node,
    destination_node: Node,
    capacity: u8,
    weight: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
