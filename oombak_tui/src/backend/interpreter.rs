use std::path::PathBuf;

use bitvec::vec::BitVec;

use crate::utils::bitvec_str;

pub enum Command {
    Run(u64),
    Load(PathBuf),
    Set(String, BitVec<u32>),
    Quit,
    Help,
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
        "set" => parse_set(args),
        "quit" => Ok(Command::Quit),
        "help" => Ok(Command::Help),
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
        Err("expected 1 argument (sv_path: String)".to_string())
    } else {
        Ok(Command::Load(PathBuf::from(args[0])))
    }
}

fn parse_set(args: &[&str]) -> Result<Command, String> {
    if args.len() != 2 {
        return Err("expected 2 argument (sigal_name: String, value: String)".to_string());
    }
    match bitvec_str::parse(args[1]) {
        Ok(value) => Ok(Command::Set(args[0].to_string(), value)),
        Err(e) => Err(e),
    }
}
