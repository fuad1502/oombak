use std::{path::PathBuf, sync::OnceLock};

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

struct CommandInfo {
    name: &'static str,
    description: &'static str,
    args: Vec<&'static str>,
    parser: Parser,
}

type Parser = Box<dyn Fn(&[&str]) -> Result<Command, String> + Send + Sync>;

static ALL_COMMAND_INFO: OnceLock<[CommandInfo; 5]> = OnceLock::new();

fn all_command_info() -> &'static [CommandInfo; 5] {
    ALL_COMMAND_INFO.get_or_init(|| {
        [
            CommandInfo {
                name: "run",
                description: "",
                args: vec!["duration"],
                parser: Box::new(parse_run),
            },
            CommandInfo {
                name: "load",
                description: "",
                args: vec!["SystemVerilog file path"],
                parser: Box::new(parse_load),
            },
            CommandInfo {
                name: "set",
                description: "",
                args: vec!["signal name", "value"],
                parser: Box::new(parse_set),
            },
            CommandInfo {
                name: "quit",
                description: "",
                args: vec![],
                parser: Box::new(parse_quit),
            },
            CommandInfo {
                name: "help",
                description: "",
                args: vec![],
                parser: Box::new(parse_help),
            },
        ]
    })
}

pub fn interpret(command_string: &str) -> Result<Command, String> {
    let words: Vec<&str> = command_string.split_whitespace().collect();
    if words.is_empty() {
        return Ok(Command::Noop);
    }
    let (command, args) = words.split_at(1);
    for command_info in all_command_info() {
        if command_info.name == command[0] {
            check_usage(command_info, args)?;
            return (command_info.parser)(args);
        }
    }
    Err(format!("unknown command \"{}\"", command[0]))
}

fn parse_run(args: &[&str]) -> Result<Command, String> {
    if let Ok(duration) = args[0].parse() {
        Ok(Command::Run(duration))
    } else {
        Err(format!("cannot parse {} as u64", args[0]))
    }
}

fn parse_load(args: &[&str]) -> Result<Command, String> {
    Ok(Command::Load(PathBuf::from(args[0])))
}

fn parse_set(args: &[&str]) -> Result<Command, String> {
    match bitvec_str::parse(args[1]) {
        Ok(value) => Ok(Command::Set(args[0].to_string(), value)),
        Err(e) => Err(e),
    }
}

fn parse_quit(_args: &[&str]) -> Result<Command, String> {
    Ok(Command::Quit)
}

fn parse_help(_args: &[&str]) -> Result<Command, String> {
    Ok(Command::Quit)
}

fn check_usage(command_info: &CommandInfo, args: &[&str]) -> Result<(), String> {
    if args.len() != command_info.args.len() {
        return Err(format!(
            "expected {} arguments (usage: {})",
            command_info.args.len(),
            command_info.usage()
        ));
    }
    Ok(())
}

impl CommandInfo {
    fn usage(&self) -> String {
        let mut usage = self.name.to_string();
        for arg in self.args.iter() {
            usage += " <";
            usage += arg;
            usage += ">";
        }
        usage
    }
}
