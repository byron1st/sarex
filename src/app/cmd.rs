use super::{config, model::*};
use clap::{Parser, Subcommand};
use log::{error, info};
use std::{error::Error, fmt::Display, fs::OpenOptions, io::BufRead, io::BufReader, path::Path};

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
        /// A file path that contains all dependency relations
        file: String,

        #[arg(short, long)]
        /// A root package or directory of the target software
        source: String,
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
    },
}

#[derive(Debug)]
enum CmdError {
    NotEnoughArguments,
    NoSuchProject,
    NoSuchFile,
}

impl Error for CmdError {}

impl Display for CmdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CmdError::NotEnoughArguments => write!(f, "Not enough arguments"),
            CmdError::NoSuchProject => write!(f, "No such project"),
            CmdError::NoSuchFile => write!(f, "No such file"),
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
        Some(Commands::Dr { file, source }) => filter_drs(file, source).await,
        Some(Commands::Ci {
            execution_traces,
            output_file,
        }) => {
            info!(
                "execution_traces: {}, output_file: {}",
                execution_traces, output_file
            );
            Ok(())
        }
        Some(Commands::Conn {
            ci_file,
            output_file,
        }) => {
            info!("file: {}, target: {}", ci_file, output_file);
            Ok(())
        }
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
                created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
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

async fn filter_drs(file: String, source: String) -> Result<(), Box<dyn Error>> {
    let p = Path::new(&file);
    if !Path::exists(p) {
        return Err(Box::new(CmdError::NoSuchFile));
    }

    let config = config::read()?;
    let file = OpenOptions::new().read(true).open(p)?;
    let reader = BufReader::new(file);

    let mut drs: Vec<drs::Dr> = Vec::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            match serde_json::from_str::<drs::Dr>(&line) {
                Ok(dr) => {
                    if !dr.target.starts_with(&source) && dr.source.starts_with(&source) {
                        drs.push(dr);
                    }
                }
                Err(e) => return Err(Box::new(e)),
            };
        }
    }

    drs::create_many(&config.db_url, drs).await
}
