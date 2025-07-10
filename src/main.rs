use serde::Deserialize;
use std::{error::Error, process};

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    #[serde(skip)]
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
fn main() -> Result<(), Box<dyn Error>> {
    fn load_network_links() -> Result<Vec<Link>, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path("configuration/network.csv")?;
        let mut network_links: Vec<Link> = Vec::new();

        for network in rdr.deserialize() {
            let loaded_network: Link = network?;
            network_links.push(loaded_network);
        }
        Ok(network_links)
    }

    match load_network_links() {
        Ok(networks) => println!("{:?}", networks),
        Err(e) => println!("An error occured: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use crate::Link;
    #[test]
    fn network_load_succesfull() {}
}
