use clap::Parser;
use std::path::PathBuf;

mod local_reader;
mod pipelines; // Make sure your pipelines module is imported

#[derive(Parser)]
struct Cli {
    #[clap(long)]
    database_url: String,
    
    #[clap(long)]
    checkpoint_dir: Option<PathBuf>,
    
    #[clap(long)]
    local_mode: bool,
    
    #[clap(long)]
    remote_store_url: Option<String>,
    
    #[clap(long)]
    first_checkpoint: Option<u64>,
    
    #[clap(long)]
    skip_watermark: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if cli.local_mode {
        let checkpoint_dir = cli.checkpoint_dir
            .ok_or_else(|| anyhow::anyhow!("Checkpoint directory must be specified in local mode"))?;
        
        println!("Starting indexer in local mode, reading checkpoints from: {:?}", checkpoint_dir);
        
        // Run the local indexer
        local_reader::run_local_indexer(
            checkpoint_dir,
            &cli.database_url,
            5, // concurrency
            PathBuf::from("/tmp/indexer_progress"),
        ).await?;
    } else {
        // Your existing remote reader logic
        println!("Starting indexer in remote mode");
        // ...
    }
    
    Ok(())
}