use super::mongo::{
    create_new_project, get_mongo_client, read_project_by_id, read_projects, update_project_name,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    error::Error,
    fmt::Display,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Debug)]
enum DBError {
    NotEnoughArguments,
    NoSuchProject,
}

impl Error for DBError {}

impl Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::NotEnoughArguments => write!(f, "Not enough arguments"),
            DBError::NoSuchProject => write!(f, "No such project"),
        }
    }
}

pub async fn set_db(db_url: String) -> Result<(), Box<dyn Error>> {
    get_mongo_client(&db_url).await?; // Check if the URL is valid

    let mut config = read_config()?;

    config.db_url = db_url;

    let p = get_path()?;
    write_config(&config, &p)?;

    Ok(())
}

pub async fn get_db() -> Result<(), Box<dyn Error>> {
    let config = read_config()?;

    let mut s = String::new();

    if config.db_url != "" {
        s.push_str(&format!("db_url: {}\n", config.db_url));
    } else {
        s.push_str("db_url: <NOT SET>\n");
    }

    let project_id = match config.project_id {
        Some(id) => id,
        None => "".to_string(),
    };

    s.push_str(&format!("project_id: {}\n", &project_id));

    if config.db_url != "" {
        let projects = read_projects(&config.db_url).await?;

        s.push_str("projects:\n");
        for project in projects {
            let id = match project.id {
                Some(id) => id.to_hex(),
                None => "".to_string(),
            };
            let created_at = project.created_at.to_chrono();
            let checked = if project_id == id { "V" } else { "-" };

            s.push_str(&format!(
                "    {} {}: {}, {}\n",
                checked,
                id,
                project.name,
                created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ));
        }
    }

    println!("{}", s);
    Ok(())
}

pub async fn set_project(
    project_id: Option<String>,
    name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut config = read_config()?;

    let id = match (project_id, name) {
        (Some(id), Some(name)) => update_project_name(&config.db_url, &id, name).await?,
        (Some(id), None) => match read_project_by_id(&config.db_url, &id).await? {
            Some(_) => id,
            None => return Err(Box::new(DBError::NoSuchProject)),
        },
        (None, Some(name)) => create_new_project(&config.db_url, name).await?,
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
