use crate::graph::Graph;

pub struct Context<'a> {
    pub graph: &'a Graph,
    pub nodes: Vec<usize>,
}

impl<'a> Context<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Context {
            nodes:graph.nodes.keys().copied().collect(),
            graph,
        }
    }
}