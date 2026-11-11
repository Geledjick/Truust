use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File,
    Dir,
    Link {target: PathBuf},
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub children: Vec<usize>,
}

pub struct Graph {
    pub root_id: usize,
    pub nodes: Vec<Node>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            root_id: 0,
            nodes: Vec::new(),
        }
    }

    pub fn from_dir(mut self, root: &Path) -> io::Result<Graph> {
        let root = fs::canonicalize(root)?;
        let metadata = fs::symlink_metadata(&root)?;

        if !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Target path is not a directory!"
            ));
        }

        let root_id = self.add_node(Node {
            name: root.file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            node_type: NodeType::Dir,
            children: Vec::new(),
        });

        self.root_id = root_id;
        
        Graph::visit_dir(&mut self, root_id, &root)?;
        
        Ok(self)
    }

    fn visit_dir(graph: &mut Graph, dir_id: usize, dir_path: &Path) -> io::Result<()> {
        let mut entries = fs::read_dir(dir_path)?
            .collect::<Result<Vec<_>, io::Error>>()?;

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            let file_type = metadata.file_type();

            let is_dir = file_type.is_dir();
            
            let node_type = if file_type.is_symlink() {
                let target = fs::read_link(&path)?;
                let resolved = if target.is_absolute() {
                    target
                } else {
                    path.parent()
                        .unwrap_or(Path::new("."))
                        .join(&target)
                };

                NodeType::Link {target: resolved}
            } else if is_dir {
                NodeType::Dir
            } else {
                NodeType::File
            };

            let child_id = graph.add_node(Node {
                name: entry.file_name()
                    .to_string_lossy()
                    .into_owned(),
                node_type,
                children: Vec::new(),
            });

            graph.nodes[dir_id].children.push(child_id);

            if is_dir {
                Graph::visit_dir(graph, child_id, &path)?;
            }
        }

        Ok(())
    }

    fn add_node(&mut self, node: Node) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn print_tree(&self, id: usize, prefix: &str) {
        if id == self.root_id {
            println!("{}/", self.nodes[id].name);
        }

        let children = &self.nodes[id].children;

        for (i, &child_id) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            let branch = if is_last {"└── "} else {"├── "};
            let node = &self.nodes[child_id];

            let suffix = match &node.node_type {
                NodeType::Dir => "/",
                NodeType::Link { .. } => " ->",
                NodeType::File => "",
            };

            println!("{}{}{}{}", prefix, branch, node.name, suffix);

            if node.node_type == NodeType::Dir {
                let new_prefix = format!("{}{}", prefix, if is_last {"    "} else {"│   "});
                self.print_tree(child_id, &new_prefix);
            }
        }
    }
}