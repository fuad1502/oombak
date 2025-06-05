use std::{path::PathBuf, sync::OnceLock};

use bitvec::vec::BitVec;

use crate::utils::bitvec_str;

pub enum Command {
    Run(usize),
    Load(PathBuf),
    Set(String, BitVec<u32>),
    SetPeriodic(String, usize, BitVec<u32>, BitVec<u32>),
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

static ALL_COMMAND_INFO: OnceLock<[CommandInfo; 6]> = OnceLock::new();
static HELP: OnceLock<String> = OnceLock::new();

fn all_command_info() -> &'static [CommandInfo; 6] {
    ALL_COMMAND_INFO.get_or_init(|| {
        [
            CommandInfo {
                name: "run",
                description: "run the simulation for as long as the duration.",
                args: vec!["duration"],
                parser: Box::new(parse_run),
            },
            CommandInfo {
                name: "load",
                description: "loads the file for simulation",
                args: vec!["SystemVerilog file path"],
                parser: Box::new(parse_load),
            },
            CommandInfo {
                name: "set",
                description: "sets the signal value",
                args: vec!["signal name", "value"],
                parser: Box::new(parse_set),
            },
            CommandInfo {
                name: "set-periodic",
                description: "set periodic signal value",
                args: vec![
                    "signal name",
                    "period",
                    "low state value",
                    "high state value",
                ],
                parser: Box::new(parse_set_periodic),
            },
            CommandInfo {
                name: "quit",
                description: "closes this application",
                args: vec![],
                parser: Box::new(parse_quit),
            },
            CommandInfo {
                name: "help",
                description: "displays this message",
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

pub fn help() -> &'static str {
    HELP.get_or_init(|| {
        let mut help = "Commands:\n".to_string();
        for command_info in all_command_info() {
            help += "   ";
            help += &command_info.usage();
            help += "\n";
            help += "       ";
            help += command_info.description;
            help += "\n";
        }
        help
    })
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

fn parse_set_periodic(args: &[&str]) -> Result<Command, String> {
    match (
        args[1].parse::<usize>(),
        bitvec_str::parse(args[2]),
        bitvec_str::parse(args[3]),
    ) {
        (Ok(period), Ok(low_value), Ok(high_value)) => Ok(Command::SetPeriodic(
            args[0].to_string(),
            period,
            low_value,
            high_value,
        )),
        (Err(e), _, _) => Err(e.to_string()),
        (_, Err(e), _) => Err(e),
        (_, _, Err(e)) => Err(e),
    }
}

fn parse_quit(_args: &[&str]) -> Result<Command, String> {
    Ok(Command::Quit)
}

fn parse_help(_args: &[&str]) -> Result<Command, String> {
    Ok(Command::Help)
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
