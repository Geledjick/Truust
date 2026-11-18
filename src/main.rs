mod graph; use graph::Graph;
mod context; use context::Context;
mod step;
mod command; use command::Command;

use std::fs;
use std::io;
use std::env;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args()
        .skip(1)
        .collect();
    
    let command = Command::parse(&args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let root = fs::canonicalize(&command.root)?;

    let graph = if command.need_graph {
        Graph::new().from_dir(&root)?
    } else {
        Graph::new()
    };
    
    let mut ctx = Context::new(graph, root);
    for step in &command.steps {
        step.run(&mut ctx);
    }
    
    Ok(())
}