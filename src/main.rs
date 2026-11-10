mod graph; use graph::Graph;
mod command; use command::{Action, Output, Command};
use std::fs;
use std::io;
use std::env;

use crate::graph::NodeType;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args()
        .skip(1)
        .collect();

    let command = Command::parse(&args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let real_root = match fs::canonicalize(command.root.clone()) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    let graph = Graph::new()
        .from_dir(&command.root)?;
    
    match command.action {
        Action::Print {outputs} => {
            graph.print_tree(0);

            for output in &outputs {
                match output {
                    Output::Amount => {
                        let amount = graph.amount_of();
                        println!("Files: {}\t Directories: {}", amount.0, amount.1);
                    }
                    
                    _ => {}
                } 
            }
        }
        Action::Amount {outputs} => {
            let amount = graph.amount_of();
            println!("Files: {}\t Directories: {}", amount.0, amount.1);

            for output in &outputs {
                match output {
                    Output::Print => graph.print_tree(0),

                    _ => {}
                }
            }
        }
        Action::Find {query, outputs} => {
            println!("Searching for: {}", query);

            let results = graph.find(&query);

            if results.is_empty() {
                println!("Nothing found :(");
            } else {
                for output in &outputs {
                    match output {
                        Output::Print => {
                            println!("Displaying search results:");
                            
                            for id in &results {
                                if let Some(node) = graph.nodes.get(&id) {
                                    if let Ok(relative) = node.path.strip_prefix(&real_root) {
                                        println!("\t{}", relative.display());
                                    }
                                }
                            }                  
                        }
                        Output::Amount => {
                            let mut amount_of_files: usize = 0;
                            let mut amount_of_directories: usize = 0;
    
                            for id in &results {
                                if let Some(node) = graph.nodes.get(&id) {
                                    match node.node_type {
                                        NodeType::File => amount_of_files += 1,
                                        NodeType::Directory => amount_of_directories += 1,
                                    }
                                }
                            }
    
                            println!("Files: {}\t Directories: {}", amount_of_files, amount_of_directories);
                        }
                    } 
                }
            }
        }
    }
    
    Ok(())
}