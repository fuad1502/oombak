pub enum Command {
    Run(u64),
    Load(String),
    Noop,
}

pub fn interpret(command_string: &str) -> Result<Command, String> {
    let words: Vec<&str> = command_string.split_whitespace().collect();
    if words.is_empty() {
        return Ok(Command::Noop);
    }
    let (command, args) = words.split_at(1);
    match command[0] {
        "run" => parse_run(args),
        "load" => parse_load(args),
        _ => Err(format!("unknown command \"{}\"", command[0])),
    }
}

fn parse_run(args: &[&str]) -> Result<Command, String> {
    if args.len() != 1 {
        return Err("expected 1 argument (duration: u64)".to_string());
    }
    if let Ok(duration) = args[0].parse() {
        Ok(Command::Run(duration))
    } else {
        Err(format!("cannot parse {} as u64", args[0]))
    }
}

fn parse_load(args: &[&str]) -> Result<Command, String> {
    if args.len() != 1 {
        Err("expected 1 argument (lib_path: String)".to_string())
    } else {
        Ok(Command::Load(args[0].to_string()))
    }
}
