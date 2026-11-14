use std::collections::HashMap;
use std::io;

use crate::context::Context;
use crate::graph::{Graph, NodeType};

use globset::Glob;

pub trait Step {
    fn run(&self, ctx: &mut Context);
    fn is_output(&self) -> bool {
        false
    }
}

// Parser
pub type Parser = fn(&mut std::slice::Iter<String>) -> Result<Box<dyn Step>, String>;
pub fn registry() -> HashMap<&'static str, Parser> {
    let mut map: HashMap<&'static str, Parser> = HashMap::new();

    // actions
    map.insert("find", |args| {
        let q = args.next().ok_or("find requires a query")?;
        Glob::new(q).map_err(|e| format!("Invalid glob: {}", e))?;
        Ok(Box::new(Find { query: q.clone() }))
    });
    map.insert("remove", |args| {
        let n = args.next().ok_or("remove requires a name")?;
        Ok(Box::new(Remove { name: n.clone() }))
    });
    map.insert("clear", |_| Ok(Box::new(Clear)));

    // outputs
    map.insert("print", |_| Ok(Box::new(Print)));
    map.insert("tree", |_| Ok(Box::new(Tree)));
    map.insert("amount", |_| Ok(Box::new(Amount)));

    // helpers
    map.insert("help", |_| Ok(Box::new(Help)));
    map.insert("version", |_| Ok(Box::new(Version)));
    
    map
}

// Actions
pub struct Find {pub query: String}
impl Step for Find {
    fn run(&self, ctx: &mut Context) {
        let matcher = Glob::new(&self.query)
            .expect("Invalid glob pattern")
            .compile_matcher();

        let has_separator = self.query.contains('/');

        ctx.graph = ctx.graph.subgraph(&|node| {
           if has_separator {
               matcher.is_match(&node.path)
           } else {
               matcher.is_match(&node.name)
           }
        }, false);
    }
}

pub struct Remove {pub name: String}
impl Step for Remove {
    fn run(&self, ctx: &mut Context) {
        ctx.graph = ctx.graph.subgraph(&|node| {
            node.name != self.name
        }, true);
    }
}

pub struct Clear;
impl Step for Clear {
    fn run(&self, ctx: &mut Context) {
        ctx.graph = Graph::new();
    }

    fn is_output(&self) -> bool {
        true
    }
}

// Outputs
pub struct Print;
impl Step for Print {
    fn run(&self, ctx: &mut Context) {
        for node in &ctx.graph.nodes {
            match &node.node_type {
                NodeType::Dir { .. } => continue,
                NodeType::Link { .. } => println!("{} -> ...", node.name),
                NodeType::File { .. } => println!("{}", node.name),
            }
        }
    }

    fn is_output(&self) -> bool {
        true
    }
}

pub struct Tree;
impl Step for Tree {
    fn run(&self, ctx: &mut Context) {
        if let Err(e) = ctx.graph.print_tree(ctx.graph.root_id, "") {
            if e.kind() != io::ErrorKind::BrokenPipe {
                eprintln!("{}", e);
            }
        }
    }

    fn is_output(&self) -> bool {
        true
    }
}

pub struct Amount;
impl Step for Amount {
    fn run(&self, ctx: &mut Context) {
        let mut files = 0;
        let mut dirs = 0;
        let mut links = 0;

        for node in &ctx.graph.nodes {
            match node.node_type {
                NodeType::File { .. } => files += 1,
                NodeType::Dir { .. } => dirs += 1,
                NodeType::Link { .. } => links += 1,
            }
        }

        println!("Files: {}  Dirs: {}  Symlinks: {}", files, dirs, links);
    }

    fn is_output(&self) -> bool {
        true
    }
}

// Helpers
pub struct Help;
impl Step for Help {
    fn run(&self, _: &mut Context) {
        println!("\
            Truust 🌲 — a tree alternative with a pipeline CLI
            
            USAGE:
                truust [path] <command>...
            
                If path is omitted, current directory is used.
                Commands are chained into a pipeline — each one
                transforms or reads the current state.
            
            COMMANDS:
                Filter:
                    find <pattern>      Keep nodes matching glob pattern
                    remove <pattern>    Remove nodes matching glob pattern
            
                Output:
                    tree                Display results as a tree
                    print               List results flat
                    amount              Count files, dirs and symlinks
                    help                Show this message
                    version             Show version of Truust

                Other:
                    clear               Clear current graph.
            
            EXAMPLES:
                truust tree
                truust ~/projects tree
                truust find \"*.rs\" tree
                truust remove \"target\" remove \".git\" tree
                truust find \"*.rs\" amount
                truust remove \"target\" find \"*.rs\" tree amount
            
            GLOB PATTERNS:
                *.rs                Any .rs file
                *.{{rs,toml}}         .rs or .toml files
                main*               Anything starting with main
                src/*.rs            .rs files inside src/
                **/tests/*.rs       .rs files in any tests/ dir
            
                Patterns with /   match against full path
                Patterns without / match against filename only
        ...");
    }

    fn is_output(&self) -> bool {
        true
    }
}

pub struct Version;
impl Step for Version {
    fn run(&self, _: &mut Context) {
        println!("Truust's version is - {}", env!("CARGO_PKG_VERSION"));
    }

    fn is_output(&self) -> bool {
        true
    }
}