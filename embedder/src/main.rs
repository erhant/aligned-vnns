use clap::{Parser, Subcommand};
use zkvdb_embedder::*;

#[derive(Parser)]
#[command(name = "embedder")]
#[command(about = "A tool to generate and query embeddings", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const DEFAULT_MODEL: &str = "nomic-embed-text:latest";

#[derive(Subcommand)]
enum Commands {
    /// Index data at given path and generate embeddings
    Index {
        #[arg(short, long, help = "Path to the data file")]
        path: String,
        #[arg(short, long, help = "Model to use for embedding generation")]
        model: Option<String>,
    },
    /// Generate embeddings from a text, can be piped to `pbcopy`
    Query {
        #[arg(short, long, help = "Text to generate embedding for")]
        text: String,
        #[arg(short, long, help = "Model to use for embedding generation")]
        model: Option<String>,
    },
    /// Export embedded data to a Rust constant vector
    Export {
        #[arg(short, long, help = "Path to the embedded data file")]
        path: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Index { path, model } => {
            index(path, &model.clone().unwrap_or(DEFAULT_MODEL.to_string())).await;
        }
        Commands::Query { text, model } => {
            query(text, &model.clone().unwrap_or(DEFAULT_MODEL.to_string())).await;
        }
        Commands::Export { path } => {
            export(path).await;
        }
    }
}
