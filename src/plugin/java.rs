use crate::model::drs::Dr;
use std::{error::Error, fmt::Display, process::Command};

#[derive(Debug)]
enum PluginError {
    WrongArguments,
}

impl Error for PluginError {}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::WrongArguments => write!(f, "Wrong arguments"),
        }
    }
}

pub fn read_drs(project_id: &str, params: Vec<String>) -> Result<Vec<Dr>, Box<dyn Error>> {
    if params.len() != 1 {
        return Err(Box::new(PluginError::WrongArguments));
    }

    let output = Command::new("jdeps")
        .arg("-v")
        .arg("-apionly")
        .arg(&params[0])
        .output()?;
    let result = String::from_utf8_lossy(&output.stdout);

    let mut drs = Vec::new();
    for line in result.lines() {
        let tokens = line.split(" -> ").collect::<Vec<_>>();
        if tokens.len() != 2 {
            continue;
        }
        let source = tokens[0].replace(" ", "");

        let target_tokens = tokens[1].split(" ").collect::<Vec<_>>();
        if target_tokens.len() == 0 {
            continue;
        }
        let target = target_tokens[0].to_string();

        drs.push(Dr {
            id: None,
            source,
            target,
            project_id: String::from(project_id),
        });
    }

    Ok(drs)
}
