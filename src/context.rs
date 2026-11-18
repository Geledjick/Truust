use std::path::PathBuf;

use crate::graph::Graph;

pub struct Context {
    pub graph: Graph,
    pub root: PathBuf,
}
impl Context {
    pub fn new(graph: Graph, root: PathBuf) -> Context {
        Context { graph, root }
    }
}