use crate::context::Context;
use crate::graph::NodeType;

pub trait Step {
    fn run(&self, ctx: &mut Context);
}

pub struct Find {pub query: String}
impl Step for Find {
    fn run(&self, ctx: &mut Context) {
        ctx.nodes.retain(|&id| {
            ctx.graph.nodes[id].name.contains(&self.query)
        });
    }
}

pub struct Print;
impl Step for Print {
    fn run(&self, ctx: &mut Context) {
        for &id in &ctx.nodes {
            let node = &ctx.graph.nodes[id];
            let suffix = match node.node_type {
                NodeType::Dir => "/",
                NodeType::Link { .. } => " -> ...",
                NodeType::File => "",
            };
            println!("{}{}", node.name, suffix);
        }
    }
}

pub struct Tree;
impl Step for Tree {
    fn run(&self, ctx: &mut Context) {
        ctx.graph.print_tree(ctx.graph.root_id, "");
    }
}

pub struct Amount;
impl Step for Amount {
    fn run(&self, ctx: &mut Context) {
        let mut files = 0;
        let mut dirs  = 0;
        let mut links = 0;

        for &id in &ctx.nodes {
            match ctx.graph.nodes[id].node_type {
                NodeType::File => files += 1,
                NodeType::Dir => dirs  += 1,
                NodeType::Link { .. } => links += 1,
            }
        }

        println!("Files: {}  Dirs: {}  Symlinks: {}", files, dirs, links);
    }
}