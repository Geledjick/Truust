use crate::graph::Graph;

pub struct Context<'a> {
    pub graph: &'a Graph,
    pub nodes: Vec<usize>,
}

impl<'a> Context<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        let nodes = (0..graph.nodes.len())
            .filter(|&id| id != graph.root_id)
            .collect();

        Context {graph, nodes}
    }
}