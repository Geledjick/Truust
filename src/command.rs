use std::path::PathBuf;

use crate::step::{Tree, Amount, Print, Step, registry};

pub struct Command {
    pub root: PathBuf,
    pub steps: Vec<Box<dyn Step>>,
}
impl Command {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        // Just truust
        if args.is_empty() {
            return Ok(Command {
                root: PathBuf::from("."),
                steps: vec![
                    Box::new(Tree),
                    Box::new(Amount)
                ],
            });
        }

        // Another
        let registry = registry();

        let (root, args) = if registry.contains_key(args[0].as_str()) {
            (PathBuf::from("."), &args[0..])
        } else {
            (PathBuf::from(&args[0]), &args[1..])
        };

        let mut steps: Vec<Box<dyn Step>> = Vec::new();
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            let parser = registry.get(arg.as_str())
                .ok_or_else(|| format!("Unknown command: {}", arg))?;

            steps.push(parser(&mut iter)?);
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