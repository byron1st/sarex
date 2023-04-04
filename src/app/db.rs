use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub fn set_db(db_url: String) -> Result<(), Box<dyn Error>> {
    let mut config = read_config()?;

    config.db_url = db_url;

    let p = get_path()?;
    write_config(&config, &p)?;

    Ok(())
}

pub fn get_db() -> Result<(), Box<dyn Error>> {
    let config = read_config()?;

    if config.db_url != "" {
        println!("    db_url: {}", config.db_url);
    } else {
        println!("    db_url: <NOT SET>");
    }

    if let Some(project_id) = config.project_id {
        println!("project_id: {:?}", project_id);
    } else {
        println!("project_id: <NOT SET>");
    }

    // TODO: Read projects if db_url is set

    Ok(())
}

pub fn set_project(project_id: String) -> Result<(), Box<dyn Error>> {
    let mut config = read_config()?;

    config.project_id = Some(project_id);

    let p = get_path()?;
    write_config(&config, &p)?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    db_url: String,
    project_id: Option<String>,
}

const SAREX_DIR: &str = ".sarex";
const CONFIG_FILE: &str = "config.json";

fn read_config() -> Result<Config, Box<dyn Error>> {
    let p = get_path()?;

    let config = if !Path::exists(&p) {
        create_new_config(&p)?
    } else {
        read_existing_config(&p)?
    };

    Ok(config)
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

fn create_new_config(p: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let config = Config {
        db_url: String::from(""),
        project_id: None,
    };

    write_config(&config, p)?;

    Ok(config)
}

fn write_config(c: &Config, p: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new().write(true).create(true).open(p)?;
    file.write_all(serde_json::to_string(&c)?.as_bytes())?;

    Ok(())
}

fn read_existing_config(p: &PathBuf) -> Result<Config, Box<dyn Error>> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(p)?;

    let config = serde_json::from_reader(file)?;
    Ok(config)
}
