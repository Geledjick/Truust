use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File { size: u64 },
    Dir { children: Vec<usize> },
    Link { target: PathBuf },
}

#[derive(Debug)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub path: PathBuf,
}
impl Node {
    pub fn clone(&self) -> Node {
        Node {
            name: self.name.clone(),
            path: self.path.clone(),
            node_type: match &self.node_type {
                NodeType::Dir { .. } => NodeType::Dir { children: Vec::new() },
                other => other.clone(),
            },
        }
    }
}

#[derive(Debug)]
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

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn subgraph(&self, predicate: &impl Fn(&Node) -> bool, filter_dirs: bool) -> Graph {
        if self.is_empty() {
            return Graph::new();
        }

        let mut new_graph = Graph::new();

        let root = &self.nodes[self.root_id];
        let new_root_id = new_graph.add_node(root.clone());

        new_graph.root_id = new_root_id;

        let mut stack: Vec<(usize, usize)> = vec![(self.root_id, new_root_id)];
        
        while let Some((old_id, new_parent_id)) = stack.pop() {
            let children = match &self.nodes[old_id].node_type {
                NodeType::Dir { children } => children.clone(),
                _ => continue,
            };
            
            for &child_id in &children {
                let child = &self.nodes[child_id];

                match child.node_type {
                    NodeType::Dir { .. } => {
                        let keep = if filter_dirs {
                            predicate(child)
                        } else {
                            true
                        };
                        
                        if keep {
                            let new_id = new_graph.add_node(child.clone());
                            if let NodeType::Dir { children } = &mut new_graph.nodes[new_parent_id].node_type {
                                children.push(new_id);
                            }
                            stack.push((child_id, new_id));
                        }
                    }
                    NodeType::File { .. } | NodeType::Link { .. } => {
                        if predicate(child) {
                            let new_id = new_graph.add_node(child.clone());
                            if let NodeType::Dir { children } = &mut new_graph.nodes[new_parent_id].node_type {
                                children.push(new_id);
                            }
                        }
                    }
                }
            }
        }

        new_graph.prune_empty_dirs(new_root_id);
        new_graph
    }

    fn prune_empty_dirs(&mut self, id: usize) -> bool {
        let children = match &self.nodes[id].node_type {
            NodeType::Dir { children } => children.clone(),
            _ => return false,
        };

        let mut kept = Vec::new();
        for child_id in children {
            let is_dir = matches!(self.nodes[child_id].node_type, NodeType::Dir { .. });

            if is_dir {
                let has_content = self.prune_empty_dirs(child_id);
                if has_content {
                    kept.push(child_id);
                }
            } else {
                kept.push(child_id);
            }
        }

        if let NodeType::Dir { children } = &mut self.nodes[id].node_type {
            *children = kept;
            !children.is_empty()
        } else {
            false
        }
    }

    pub fn from_dir(mut self, root: &Path) -> io::Result<Graph> {
        let root = fs::canonicalize(root)?;
        let metadata = fs::symlink_metadata(&root)?;

        if !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Target path is not a directory!",
            ));
        }

        let root_id = self.add_node(Node {
            name: root.file_name().unwrap().to_string_lossy().into_owned(),
            node_type: NodeType::Dir {children: Vec::new()},
            path: root.clone(),
        });

        self.root_id = root_id;

        Graph::visit_dir(&mut self, root_id, &root)?;

        Ok(self)
    }

    fn visit_dir(graph: &mut Graph, root_id: usize, root_path: &Path) -> io::Result<()> {
        let mut stack: Vec<(usize, PathBuf)> = vec![(root_id, root_path.to_path_buf())];

        while let Some((dir_id, dir_path)) = stack.pop() {
            let mut entries: Vec<_> = match fs::read_dir(&dir_path) {
                Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
                Err(e) => {
                    eprintln!("Skip {}: {}", dir_path.display(), e);
                    continue;
                }
            };
    
            entries.sort_by_key(|e| e.file_name());
    
            for entry in entries {
                let path = entry.path();
                let metadata = match fs::symlink_metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                
                let file_type = metadata.file_type();
                let is_dir = file_type.is_dir();
    
                let node_type = if file_type.is_symlink() {
                    let target = fs::read_link(&path)?;
                    let resolved = if target.is_absolute() {
                        target
                    } else {
                        path.parent().unwrap_or(Path::new(".")).join(&target)
                    };
    
                    NodeType::Link {target: resolved}
                } else if is_dir {
                    NodeType::Dir {children: Vec::new()}
                } else {
                    NodeType::File {size: metadata.len()}
                };
    
                let child_id = graph.add_node(Node {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    node_type: node_type,
                    path: path.clone(),
                });
    
                if let NodeType::Dir { children } = &mut graph.nodes[dir_id].node_type {
                    children.push(child_id);
                }
                
                if is_dir {
                    stack.push((child_id, path));
                }
            }
        }
        
        Ok(())
    }

    fn add_node(&mut self, node: Node) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn print_tree(&self, id: usize, prefix: &str) -> io::Result<()> {
        if self.is_empty() {
            return Ok(());
        }
        
        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());
        self.print_tree_inner(id, prefix, &mut out);

        Ok(())
    }

    fn print_tree_inner(&self, id: usize, prefix: &str, out: &mut impl Write) {
        if id == self.root_id {
            writeln!(out, "{}/", self.nodes[id].name).unwrap();
        }

        let children = match &self.nodes[id].node_type {
            NodeType::Dir { children } => children,
            _ => return,
        };

        for (i, &child_id) in children.iter().enumerate() {
            let is_last = i == children.len() - 1;
            let branch = if is_last {"└── "} else {"├── "};
            let node = &self.nodes[child_id];

            let suffix = match node.node_type {
                NodeType::Dir { .. } => "/",
                NodeType::Link { .. } => " ->",
                NodeType::File { .. } => "",
            };

            writeln!(out, "{}{}{}{}", prefix, branch, node.name, suffix).unwrap();

            if matches!(node.node_type, NodeType::Dir { .. }) {
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                self.print_tree_inner(child_id, &new_prefix, out);
            }
        }
    }
}