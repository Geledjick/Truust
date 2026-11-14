mod graph; use graph::Graph;
mod context; use context::Context;
mod step;
mod command; use command::Command;

use std::fs;
use std::io;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args()
        .skip(1)
        .collect();
    
    let command = Command::parse(&args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let root = fs::canonicalize(&command.root)?;
    
    let graph = Graph::new().from_dir(&root)?;
    
    let mut ctx = Context::new(graph);
    for step in &command.steps {
        step.run(&mut ctx);
    }

    Ok(())
}