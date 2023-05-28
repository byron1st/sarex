use super::{config, model::*};
use crate::{
    ci, conn,
    model::{self},
    plugin,
};
use clap::{Parser, Subcommand};
use log::{error, info};

use std::{error::Error, fmt::Display};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Set the database URL
    SetDB {
        /// The database URL
        db_url: String,
    },

    /// Get the database URL, which is currently set, and projects, which are stored in the database
    GetDB {},

    /// Set the project ID
    SetProject {
        /// The project ID. If the project ID is not provided, a new project is created.
        project_id: Option<String>,

        #[arg(short, long)]
        /// The project name. If the project ID is not provided, the project name is used to create a new project. If the project name is provided, the project name is set to the project.
        name: Option<String>,
    },

    /// Filter dependency relations from source code to external libraries
    Dr {
        #[arg(short, long)]
        /// A root path for the target software
        root_path: String,

        #[arg(short, long)]
        /// A programming language of the target software. Currently, only "java", "go", and "js" are supported.
        lang: String,

        #[arg(short, long)]
        /// Packages or directories of the target software. Comma separated values are allowed.
        sources: String,
    },

    /// Extract connector instances from execution traces
    Ci {
        #[arg(short, long)]
        /// A directory path that contains all execution traces
        execution_traces: String,

        #[arg(short, long)]
        /// An output file path that contains connector instances
        output_file: String,
    },

    /// Build an execution view model from connector instances
    Conn {
        #[arg(short, long)]
        /// A file path that contains connector instances
        ci_file: String,

        #[arg(short, long)]
        /// An output file path that contains an execution view model
        output_file: String,

        #[arg(short, long)]
        /// An output format of the execution view model. Currently, "json", "png", and "dot" are supported.
        format: String,
    },
}

#[derive(Debug)]
enum CmdError {
    NotEnoughArguments,
    NoSuchProject,
    WrongArguments,
    NoProjectIdSet,
}

impl Error for CmdError {}

impl Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdError::NotEnoughArguments => write!(f, "Not enough arguments"),
            CmdError::NoSuchProject => write!(f, "No such project"),
            CmdError::WrongArguments => write!(f, "Wrong arguments"),
            CmdError::NoProjectIdSet => write!(f, "No project ID is set"),
        }
    }
}

pub async fn init_app() {
    let cli = Cli::parse();

    match run_command(cli.command).await {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
}

async fn run_command(cmd: Option<Commands>) -> Result<(), Box<dyn Error>> {
    match cmd {
        Some(Commands::SetDB { db_url }) => set_db(db_url).await,
        Some(Commands::GetDB {}) => get_db().await,
        Some(Commands::SetProject { project_id, name }) => set_project(project_id, name).await,
        Some(Commands::Dr {
            root_path,
            lang,
            sources,
        }) => save_drs(root_path, lang, sources).await,
        Some(Commands::Ci {
            execution_traces,
            output_file,
        }) => extract_cis(execution_traces, output_file).await,
        Some(Commands::Conn {
            ci_file,
            output_file,
            format,
        }) => build_connectors(ci_file, output_file, format),
        None => {
            error!("No command provided");
            Ok(())
        }
    }
}

async fn set_db(db_url: String) -> Result<(), Box<dyn Error>> {
    mongo::get_mongo_client(&db_url).await?; // Check if the URL is valid

    let mut config = config::read()?;

    config.db_url = db_url;

    config::write(&config)?;

    Ok(())
}

async fn get_db() -> Result<(), Box<dyn Error>> {
    let config = config::read()?;

    let mut s = String::new();

    if !config.db_url.is_empty() {
        s.push_str(&format!("db_url: {}\n", config.db_url));
    } else {
        s.push_str("db_url: <NOT SET>\n");
    }

    let project_id = match config.project_id {
        Some(id) => id,
        None => "".to_string(),
    };

    s.push_str(&format!("project_id: {}\n", &project_id));

    if !config.db_url.is_empty() {
        let projects = projects::read_many(&config.db_url).await?;

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
                created_at.format("%Y-%m-%d %H:%M:%S"),
            ));
        }
    }

    println!("{}", s);
    Ok(())
}

async fn set_project(
    project_id: Option<String>,
    name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut config = config::read()?;

    let id = match (project_id, name) {
        (Some(id), Some(name)) => projects::update(&config.db_url, &id, name).await?,
        (Some(id), None) => match projects::read_one(&config.db_url, &id).await? {
            Some(_) => id,
            None => return Err(Box::new(CmdError::NoSuchProject)),
        },
        (None, Some(name)) => projects::create(&config.db_url, name).await?,
        (None, None) => {
            return Err(Box::new(CmdError::NotEnoughArguments));
        }
    };

    config.project_id = Some(id);

    config::write(&config)?;

    Ok(())
}

async fn save_drs(root_path: String, lang: String, sources: String) -> Result<(), Box<dyn Error>> {
    let config = config::read()?;
    let project_id = match config.project_id {
        Some(id) => id,
        None => {
            return Err(Box::new(CmdError::NoProjectIdSet));
        }
    };

    let kind: plugin::PluginKind = match lang.as_str() {
        "java" => plugin::PluginKind::Java,
        "go" => plugin::PluginKind::Go,
        "js" => plugin::PluginKind::JavaScript,
        _ => return Err(Box::new(CmdError::WrongArguments)),
    };

    let params: Vec<&str> = vec![&root_path, &sources];

    let all_drs = plugin::read_drs(&project_id, kind, params)?;
    if all_drs.is_empty() {
        info!("No drs found");
        return Ok(());
    }

    let s = sources.split(',').collect::<Vec<_>>();

    let filtered_drs = all_drs
        .iter()
        .filter(|dr| !is_start_with(&dr.target, &s) && is_start_with(&dr.source, &s))
        .collect::<Vec<_>>();

    match drs::create_many(&config.db_url, filtered_drs).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn is_start_with(item: &str, sources: &Vec<&str>) -> bool {
    for source in sources {
        if item.starts_with(source) {
            return true;
        }
    }

    false
}

async fn extract_cis(
    execution_traces_file_path_str: String,
    output_file_path_str: String,
) -> Result<(), Box<dyn Error>> {
    let config = config::read()?;
    let project_id = config.project_id.ok_or(CmdError::NoProjectIdSet)?;

    let execution_traces = ci::read_execution_traces(execution_traces_file_path_str)?;
    let mapping_rules = model::mapping_rules::read_many(&config.db_url, &project_id).await?;

    let cis = ci::create_cis(execution_traces, mapping_rules)?;

    ci::write_cis(cis, &output_file_path_str)
}

fn build_connectors(
    ci_file: String,
    output_file: String,
    output_format: String,
) -> Result<(), Box<dyn Error>> {
    let cis = conn::read_cis(&ci_file)?;
    let model = conn::build_model(cis)?;
    conn::write_model(model, &output_file, &output_format)
}
