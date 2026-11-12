use std::path::PathBuf;

use crate::step::{Print, Step, registry};

pub struct Command {
    pub root: PathBuf,
    pub steps: Vec<Box<dyn Step>>
}
impl Command {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.is_empty() {
            return Err("No commands provided".to_string());
        }
        
        let registry = registry();

        let (root, args) = if registry.contains_key(args[0].as_str()) {
            (PathBuf::from("."), &args[0..])
        } else {
            (PathBuf::from(&args[0]), &args[1..])
        };
        
        let mut steps: Vec<Box<dyn Step>> = Vec::new();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match registry.get(arg.as_str()) {
                Some(parser) => steps.push(parser(&mut iter)?),
                None => return Err(format!("Unknown command: {}", arg)),
            }
        }

        if !steps.last().map(|s| s.is_output()).unwrap_or(false) {
            steps.push(Box::new(Print));
        }
    
        if steps.is_empty() {
            return Err("No commands provided".to_string());
        }
        
        Ok(Command {root, steps})
    }
} 