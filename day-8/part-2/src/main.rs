use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let file = File::open(filename).expect("Could not open file");
    let mut reader = BufReader::new(file);

    let mut next_id = 0;
    let mut expected_numbers = vec![ExpectedType::Header(next_id)];
    let mut nodes: HashMap<i32, Node> = HashMap::new();

    while !expected_numbers.is_empty() {
        match expected_numbers.pop() {
            Some(ExpectedType::Header(this_node)) => {
                // Read header
                let num_children =
                    fetch_next_number(&mut reader).expect("Error reading number of children");
                let num_metadata_entries = fetch_next_number(&mut reader)
                    .expect("Error reading number of metadata entries");

                // Push the metadata onto stack first, as we will read it last
                expected_numbers.append(&mut vec![
                    ExpectedType::MetadataEntry(this_node);
                    num_metadata_entries as usize
                ]);

                // Get the ids for the children for this node and increment the id tracker
                let children_ids = (next_id + 1..next_id as i32 + 1 + num_children)
                    .collect::<Vec<i32>>();
                next_id += num_children;

                // Reverse ids for stack (to keep child indexes consistent)
                let mut header_ids = children_ids.clone();
                header_ids.reverse();

                // Push expected headers onto stack
                let mut new_node_headers = header_ids
                    .iter()
                    .map(|&id| ExpectedType::Header(id))
                    .collect();
                expected_numbers.append(&mut new_node_headers);

                //Add the node to the hashmap
                nodes.insert(
                    this_node,
                    Node {
                        children: children_ids,
                        metadata: Vec::new(),
                    },
                );
            }
            Some(ExpectedType::MetadataEntry(parent)) => {
                let metadata =
                    fetch_next_number(&mut reader).expect("Error reading metadata entry");
                nodes.get_mut(&parent).unwrap().metadata.push(metadata);
            }
            None => {
                eprintln!("Unexpected end of stack");
                process::exit(1);
            }
        }
    }
    println!("{}", value_of_node(&0, &nodes));
}

#[derive(Debug, Clone)]
enum ExpectedType {
    Header(i32),
    MetadataEntry(i32),
}

#[derive(Debug)]
struct Node {
    children: Vec<i32>,
    metadata: Vec<i32>,
}

fn fetch_next_number(reader: &mut BufReader<File>) -> Result<i32, Box<Error>> {
    let mut buffer = vec![];
    // Read next number
    reader.read_until(b' ', &mut buffer)?;
    let num_string = String::from_utf8(buffer)?;
    let num = num_string.trim().parse::<i32>()?;
    Ok(num)
}

fn value_of_node(node_id: &i32, nodes: &HashMap<i32, Node>) -> i32 {
    match nodes.get(node_id) {
        Some(node) => {
            if node.children.len() > 0 {
                node.metadata
                    .iter()
                    .filter_map(|&index| node.children.get((index - 1) as usize))
                    .map(|id| value_of_node(id, nodes))
                    .sum()
            } else {
                node.metadata.iter().sum()
            }
        }
        None => 0,
    }
}
