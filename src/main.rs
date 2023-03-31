use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{error, info};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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

fn main() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Dr { file, target }) => {
            info!("file: {}, target: {}", file, target);
        }
        Some(Commands::Ci {
            execution_traces,
            output_file,
        }) => {
            info!(
                "execution_traces: {}, output_file: {}",
                execution_traces, output_file
            );
        }
        Some(Commands::Conn {
            ci_file,
            output_file,
        }) => {
            info!("file: {}, target: {}", ci_file, output_file);
        }
        None => {
            error!("No command provided");
        }
    }
}
