use clap::Parser;
use sui_indexer_alt_framework::{
    cluster::{self, IndexerCluster},
    pipeline::concurrent::ConcurrentConfig,
    Result,
};
use sui_sender_indexer::{BlobPipeline, MIGRATIONS};
use url::Url;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(
        long,
        default_value = "postgres://postgres:sui-indexer@localhost:5432/sui_sender"
    )]
    database_url: Url,

    #[clap(flatten)]
    cluster_args: cluster::Args,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut indexer =
        IndexerCluster::new(args.database_url, args.cluster_args, Some(&MIGRATIONS)).await?;

    // Comment out SenderPipeline to only index blobs
    // indexer
    //     .concurrent_pipeline(SenderPipeline, ConcurrentConfig::default())
    //     .await?;

    // Only run the BlobPipeline
    indexer
        .concurrent_pipeline(BlobPipeline, ConcurrentConfig::default())
        .await?;

    let _ = indexer.run().await?.await;
    Ok(())
}
