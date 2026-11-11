use std::path::PathBuf;
use crate::step::{Amount, Print, Find, Step, Tree};

pub struct Command {
    pub root: PathBuf,
    pub steps: Vec<Box<dyn Step>>
}

impl Command {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let root = PathBuf::from(&args[0]);
        let mut steps: Vec<Box<dyn Step>> = Vec::new();
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "print" => steps.push(Box::new(Print)),
                "tree" => steps.push(Box::new(Tree)),
                "amount" => steps.push(Box::new(Amount)),
                "find" => {
                    i += 1;
                    let q = args.get(i)
                        .ok_or("Find requires a query")?
                        .clone();

                    steps.push(Box::new(Find {query: q}));
                }

                other => return Err(format!("Unknown command: {}", other)),
            }
            i += 1;
        }

        Ok(Command {root, steps})
    }
} 