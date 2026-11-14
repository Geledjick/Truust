use crate::graph::Graph;

pub struct Context {
    pub graph: Graph,
}
impl Context {
    pub fn new(graph: Graph) -> Context {
        Context { graph }
    }
}