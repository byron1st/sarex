use std::{error::Error, fmt::Display, process::Command};

use serde::{Deserialize, Serialize};

use crate::model::drs::Dr;

use super::dir;

#[derive(Debug)]
enum PluginError {
    WrongArguments,
    NoGoFileInstalled,
    CommandError(std::io::Error),
}

impl Error for PluginError {}

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::WrongArguments => write!(f, "Wrong arguments"),
            PluginError::NoGoFileInstalled => write!(f, "No Go file installed"),
            PluginError::CommandError(e) => write!(f, "Command error: {}", e),
        }
    }
}

const PLUGIN_DIR: &str = "go";
const GO_FILE: &str = "go-dependencies-reader";

#[derive(Serialize, Deserialize, Debug)]
struct DrRecord {
    caller: String,
    callee: String,
}

pub fn read_drs(project_id: &str, params: Vec<&str>) -> Result<Vec<Dr>, Box<dyn Error>> {
    if params.len() < 2 {
        return Err(Box::new(PluginError::WrongArguments));
    }

    let root_path = params[0];
    let pkg = params[1];

    let go_file = get_go_file()?;
    let output = match Command::new(go_file)
        .arg("-main")
        .arg(pkg)
        .arg("-dir")
        .arg(root_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => return Err(Box::new(PluginError::CommandError(e))),
    };

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

fn get_go_file() -> Result<String, PluginError> {
    let mut p = dir::get_plugin_dir();
    p.push(PLUGIN_DIR);
    p.push(GO_FILE);

    match p.to_str() {
        Some(g) => Ok(String::from(g)),
        None => Err(PluginError::NoGoFileInstalled),
    }
}
