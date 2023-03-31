use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum ArgsReadError {
    NoArgs,
    WrongCommand,
    WrongCommandArgs,
}

impl Error for ArgsReadError {}

impl Display for ArgsReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgsReadError::NoArgs => write!(f, "No arguments provided"),
            ArgsReadError::WrongCommand => write!(f, "Wrong command provided"),
            ArgsReadError::WrongCommandArgs => write!(f, "Wrong command arguments provided"),
        }
    }
}

#[derive(Debug)]
pub enum Command {
    DR,
    CI,
    Conn,
}

impl Command {
    pub fn new(cmd_str: &str) -> Option<Command> {
        match cmd_str {
            "dr" => Some(Command::DR),
            "ci" => Some(Command::CI),
            "conn" => Some(Command::Conn),
            _ => None,
        }
    }

    pub fn get_flags(&self) -> [&str; 2] {
        match self {
            Command::DR => ["-f", "-t"],
            Command::CI => ["-i", "-o"],
            Command::Conn => ["-i", "-o"],
        }
    }
}

pub fn parse_args(args: Vec<String>) -> Result<(Command, Vec<String>), ArgsReadError> {
    if args.len() < 2 {
        Err(ArgsReadError::NoArgs)
    } else if args.len() < 6 {
        Err(ArgsReadError::WrongCommandArgs)
    } else {
        parse_cmd_and_flags(args)
    }
}

fn parse_cmd_and_flags(args: Vec<String>) -> Result<(Command, Vec<String>), ArgsReadError> {
    let cmd = match Command::new(&args[1]) {
        Some(cmd) => cmd,
        None => return Err(ArgsReadError::WrongCommand),
    };

    let parsed_args = parse_flags(cmd.get_flags(), &args[2..])?;

    Ok((cmd, parsed_args))
}

fn parse_flags(flags: [&str; 2], all_args: &[String]) -> Result<Vec<String>, ArgsReadError> {
    if all_args.len() % 2 != 0 || all_args.len() / 2 != flags.len() {
        return Err(ArgsReadError::WrongCommandArgs);
    }

    let mut args = vec![String::new(); flags.len()];
    for i in 0..all_args.len() / 2 {
        let index = i * 2;
        let flag_index = match flags.iter().position(|x| x == &all_args[index].as_str()) {
            Some(index) => index,
            None => {
                continue;
            }
        };
        args[flag_index] = all_args[index + 1].to_owned();
    }

    for arg in args.iter() {
        if arg.is_empty() {
            return Err(ArgsReadError::WrongCommandArgs);
        }
    }

    Ok(args)
}
