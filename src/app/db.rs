use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    error::Error,
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use super::mongo::get_mongo_client;

pub async fn set_db(db_url: String) -> Result<(), Box<dyn Error>> {
    get_mongo_client(&db_url).await?; // Check if the URL is valid

    let mut config = read_config()?;

    config.db_url = db_url;

    let p = get_path()?;
    write_config(&config, &p)?;

    Ok(())
}

pub fn get_db() -> Result<(), Box<dyn Error>> {
    let config = read_config()?;

    let mut s = String::new();

    if config.db_url != "" {
        s.push_str(&format!("    db_url: {}\n", config.db_url));
    } else {
        s.push_str("    db_url: <NOT SET>\n");
    }

    if let Some(project_id) = config.project_id {
        s.push_str(&format!("project_id: {:?}\n", project_id));
    } else {
        s.push_str("project_id: <NOT SET>\n");
    }

    // TODO: Read projects if db_url is set

    println!("{}", s);
    Ok(())
}

#[derive(Debug)]
enum DBError {
    NotEnoughArguments,
}

impl Error for DBError {}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::NotEnoughArguments => write!(f, "Not enough arguments"),
        }
    }
}

pub fn set_project(project_id: Option<String>, name: Option<String>) -> Result<(), Box<dyn Error>> {
    let mut config = read_config()?;

    let id = match (project_id, name) {
        (Some(id), Some(name)) => id,
        (Some(id), None) => id,
        (None, Some(name)) => String::from("TODO: Get project id from name"),
        (None, None) => {
            return Err(Box::new(DBError::NotEnoughArguments));
        }
    };

    config.project_id = Some(id);

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
