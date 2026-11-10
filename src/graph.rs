use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EdgeType {
    Contains,
    References,
}

#[derive(Debug, Clone)]
pub struct Node {
    name: String,
    pub node_type: NodeType,
    pub path: PathBuf,
    //size: u64,
    //modified: String,
}

#[derive(Debug, Clone)]
struct Edge {
    to: usize,
    //weight: u64,
    edge_type: EdgeType,
}

pub struct Graph {
    pub nodes: HashMap<usize, Node>,
    adjacency: HashMap<usize, Vec<Edge>>,
    next_id: usize,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn from_dir(mut self, root: &Path) -> io::Result<Graph> {
        let root = fs::canonicalize(root)?;
        let metadata = fs::symlink_metadata(&root)?;

        if !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Target path is not a directory",
            ));
        }

        let root_id = self.add_vertex(Node {
            name: root.file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            node_type: NodeType::Directory,
            path: root.clone(),
            //size: 0,
            //modified: format!("{:?}", metadata.modified()?),
        });

        let mut path_to_id = HashMap::new();
        path_to_id.insert(root.clone(), root_id);

        let mut pending_links: Vec<(usize, PathBuf)> = Vec::new();

        Self::visit_directory(
            &mut self,
            root_id,
            &root,
            &mut path_to_id,
            &mut pending_links,
        )?;

        for (link_id, target_path) in pending_links {
            if let Ok(target) = fs::canonicalize(&target_path) {
                if let Some(&target_id) = path_to_id.get(&target) {
                    self.add_edge(link_id, target_id, EdgeType::References)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                }
            }
        }

        Ok(self)
    }

    fn visit_directory(
        graph: &mut Graph,
        dir_id: usize,
        dir_path: &Path,
        path_to_id: &mut HashMap<PathBuf, usize>,
        pending_links: &mut Vec<(usize, PathBuf)>,
    ) -> io::Result<()> {
        let mut entries = fs::read_dir(dir_path)?.collect::<Result<Vec<_>, io::Error>>()?;

        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let path = entry.path();

            let metadata = fs::symlink_metadata(&path)?;
            let file_type = metadata.file_type();

            let is_symlink = file_type.is_symlink();
            let is_dir = file_type.is_dir();

            let node = Node {
                name: entry.file_name()
                    .to_string_lossy()
                    .into_owned(),

                node_type: if is_dir {
                    NodeType::Directory
                } else {
                    NodeType::File
                },

                path: path.clone(),

                //size: if is_dir { 0 } else { metadata.len() },

                //modified: metadata
                //    .modified()
                //    .map(|time| format!("{:?}", time))
                //    .unwrap_or_else(|_| String::from("unknown")),
            };

            let child_id = graph.add_vertex(node);

            graph
                .add_edge(
                    dir_id,
                    child_id,
                    //if is_dir { 0 } else { metadata.len() },
                    EdgeType::Contains,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            if is_symlink {
                if let Ok(target) = fs::read_link(&path) {
                    let resolved = if target.is_absolute() {
                        target
                    } else {
                        path.parent()
                            .unwrap_or(Path::new("."))
                            .join(target)
                    };

                    pending_links.push((child_id, resolved));
                }

                continue;
            }

            let canonical_path = fs::canonicalize(&path)?;
            path_to_id.insert(canonical_path, child_id);

            if is_dir {
                Self::visit_directory(graph, child_id, &path, path_to_id, pending_links)?;
            }
        }

        Ok(())
    }

    fn add_vertex(&mut self, node: Node) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        self.nodes
            .insert(id, node);
        self.adjacency
            .insert(id, Vec::new());

        id
    }

    fn add_edge(
        &mut self,
        from: usize,
        to: usize,
        //weight: u64,
        edge_type: EdgeType,
    ) -> Result<(), String> {
        if !self.nodes.contains_key(&from) {
            return Err(format!("Source vertex {} doesn't exist", from));
        }

        if !self.nodes.contains_key(&to) {
            return Err(format!("Destination vertex {} doesn't exist", to));
        }

        let edge = Edge {
            to,
            //weight,
            edge_type,
        };

        self.adjacency
            .get_mut(&from)
            .unwrap()
            .push(edge);

        Ok(())
    }

    pub fn find(&self, query: &str) -> Vec<usize> {
        let mut results: Vec<usize> = Vec::new();

        for (id, node) in &self.nodes {
            if node.name.contains(query) {
                results.push(*id);
            }
        }

        results
    }

    pub fn amount_of(&self) -> (usize, usize) {
        let mut amount_files: usize = 0;
        let mut amount_directories: usize = 0;
        
        for (_, node) in &self.nodes {
            match node.node_type {
                NodeType::File => amount_files += 1,
                NodeType::Directory => amount_directories += 1,
            }
        }

        (amount_files, amount_directories)
     }

    pub fn print_tree(&self, root_id: usize) {
        let Some(root) = self.nodes.get(&root_id) else {
            println!("Node {} does not exist", root_id);
            return;
        };

        println!("{}/", root.name);

        self.print_tree_children(root_id, "");
    }

    fn print_tree_children(&self, dir_id: usize, prefix: &str) {
        let children: Vec<usize> = self.adjacency
            .get(&dir_id)
            .into_iter()
            .flatten()
            .filter(|edge| edge.edge_type == EdgeType::Contains)
            .map(|edge| edge.to)
            .collect();

        for (index, child_id) in children.iter().enumerate() {
            let is_last = index == children.len() - 1;

            let Some(child) = self.nodes.get(child_id) else {
                continue;
            };

            let branch = if is_last {
                "└── "
            } else {
                "├── "
            };

            let suffix = if child.node_type == NodeType::Directory {
                "/"
            } else {
                ""
            };

            println!("{}{}{}{}", prefix, branch, child.name, suffix);

            if child.node_type == NodeType::Directory {
                let new_prefix = format!("{}{}", prefix, if is_last {"    "} else {"│   "});

                self.print_tree_children(*child_id, &new_prefix);
            }
        }
    }
}