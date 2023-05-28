use serde::{Deserialize, Serialize};

use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub db_url: String,
    pub project_id: Option<String>,
}

const SAREX_DIR: &str = ".sarex";
const CONFIG_FILE: &str = "config.json";

pub fn read() -> Result<Config, Box<dyn Error>> {
    let p = get_path()?;

    let config = if !Path::exists(&p) {
        create_new_config()?
    } else {
        read_existing_config(&p)?
    };

    Ok(config)
}

pub fn write(c: &Config) -> Result<(), Box<dyn Error>> {
    let p = get_path()?;

    let mut file = OpenOptions::new().write(true).create(true).open(p)?;
    file.write_all(serde_json::to_string(&c)?.as_bytes())?;

    Ok(())
}

fn get_path() -> Result<PathBuf, Box<dyn Error>> {
    let mut p = PathBuf::new();
    if let Some(home) = dirs::home_dir() {
        p.push(home);
        p.push(SAREX_DIR);
    }

    fs::create_dir_all(&p)?;

    p.push(CONFIG_FILE);

    Ok(p)
}

fn create_new_config() -> Result<Config, Box<dyn Error>> {
    let config = Config {
        db_url: String::from(""),
        project_id: None,
    };

    write(&config)?;

    Ok(config)
}

fn read_existing_config(p: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(p)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config = serde_json::from_str(&content)?;
    Ok(config)
}
