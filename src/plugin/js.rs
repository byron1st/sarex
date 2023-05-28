use std::{error::Error, fmt::Display, process::Command};

use serde::{Deserialize, Serialize};

use crate::model::drs::Dr;

use super::dir;

#[derive(Debug)]
enum PluginError {
    WrongArguments,
    NoJSDependencyReaderInstalled,
}

impl Error for PluginError {}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::WrongArguments => write!(f, "Wrong arguments"),
            PluginError::NoJSDependencyReaderInstalled => {
                write!(f, "No Java dependency reader installed")
            }
        }
    }
}

const PLUGIN_DIR: &str = "js";
const PROJECT_DIR: &str = "js-dependencies-reader";

#[derive(Serialize, Deserialize, Debug)]
struct DrRecord {
    caller: String,
    callee: String,
}

pub fn read_drs(project_id: &str, params: Vec<&str>) -> Result<Vec<Dr>, Box<dyn Error>> {
    if params.is_empty() {
        return Err(Box::new(PluginError::WrongArguments));
    }

    let plugin_program_dir = get_reader_dir()?;

    let output = Command::new("./run.sh")
        .env("ROOT", params[0])
        .current_dir(plugin_program_dir)
        .output()?;
    let result = String::from_utf8_lossy(&output.stdout);

    let mut drs = Vec::new();
    for line in result.lines() {
        let record: DrRecord = match serde_json::from_str(line) {
            Ok(d) => d,
            Err(_) => continue,
        };

        drs.push(Dr {
            id: None,
            source: record.caller,
            target: record.callee,
            project_id: String::from(project_id),
        });
    }

    Ok(drs)
}

fn get_reader_dir() -> Result<String, PluginError> {
    let mut p = dir::get_plugin_dir();
    p.push(PLUGIN_DIR);
    p.push(PROJECT_DIR);

    match p.to_str() {
        Some(j) => Ok(j.to_string()),
        None => Err(PluginError::NoJSDependencyReaderInstalled),
    }
}
