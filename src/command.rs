use std::path::PathBuf;

#[derive(Debug)]
pub enum Output {
    Print,
    Amount,
}

#[derive(Debug)]
pub enum Action {
    Print {
        outputs: Vec<Output>,
    },
    Amount {
        outputs: Vec<Output>,
    },
    Find {
        query: String,
        outputs: Vec<Output>,
    },
}

#[derive(Debug)]
pub struct Command {
    pub root: PathBuf,
    pub action: Action,
}

impl Command {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.len() < 2 {
            return Err("Not enough arguments".to_string());
        }

        let root = PathBuf::from(&args[0]);

        let action = match args[1].as_str() {
            "print" => {
                let mut outputs: Vec<Output> = Vec::new();

                for arg in &args[2..] {
                    match arg.as_str() {
                        "amount" => outputs.push(Output::Amount),
                        _ => return Err("Unknown output operation".to_string()),
                    };
                }
                
                Action::Print {outputs}
            },
            "find" => {
                let query = args[2].clone();
                let mut outputs: Vec<Output> = Vec::new();

                for arg in &args[3..] {
                    match arg.as_str() {
                        "print" => outputs.push(Output::Print),
                        "amount" => outputs.push(Output::Amount),
                        _ => return Err("Unknown output operation".to_string()),
                    };
                }

                Action::Find {query, outputs}
            } // find
            "amount" => {
                let mut outputs: Vec<Output> = Vec::new();

                for arg in &args[2..] {
                    match arg.as_str() {
                        "print" => outputs.push(Output::Print),
                        _ => return Err("Unknown output operation".to_string()),
                    };
                }
                
                Action::Amount {outputs}
            },

            _ => return Err("Unknown command".to_string()),
        };

        Ok(Command {root, action})
    }
}