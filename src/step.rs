use crate::context::Context;
use crate::graph::NodeType;

pub trait Step {
    fn run(&self, ctx: &mut Context);
}

pub struct Find {pub query: String}
impl Step for Find {
    fn run(&self, ctx: &mut Context) {
        ctx.nodes.retain(|id| {
            ctx.graph.nodes[id].name.contains(&self.query)
        });
    }
}

pub struct Print;
impl Step for Print {
    fn run(&self, ctx: &mut Context) {
        for id in &ctx.nodes {
            if let Some(node) = ctx.graph.nodes.get(id) {
                let suffix = if node.node_type == NodeType::Directory { "/" } else { "" };
                println!("{}{}", node.name, suffix);
            }
        }
    }
}

pub struct Amount;
impl Step for Amount {
    fn run(&self, ctx: &mut Context) {
        let files = ctx.nodes.iter()
            .filter(|id| ctx.graph.nodes[id].node_type == NodeType::File)
            .count();

        let dirs = ctx.nodes.iter()
            .filter(|id| ctx.graph.nodes[id].node_type == NodeType::Directory)
            .count();

       println!("->Files: {}\n->Directories: {}", files, dirs);
    }
}