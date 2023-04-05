use super::db::{get_db, set_db, set_project};
use clap::{Parser, Subcommand};
use log::{error, info};
use std::error::Error;

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
        target: String,
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
        Some(Commands::Dr { file, target }) => {
            info!("file: {}, target: {}", file, target);
            Ok(())
        }
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
