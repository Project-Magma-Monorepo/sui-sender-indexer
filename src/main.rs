use clap::Parser;
use sui_indexer_alt_framework::{
    cluster::{self, IndexerCluster}, pipeline::concurrent::ConcurrentConfig, Result, pipeline::sequential::SequentialConfig
};
use sui_sender_indexer::{BlobPipeline,SenderPipeline, BlobIdPipeline, MIGRATIONS};
use url::Url;



// add IndexerArgs with first_checkpoint and last_checkpoint to check if a blob that you checked on Walrus explorer is well indexed 

// 153130834 checkpoint for blob 0xf9e021f91a763eba3b194d03ece28f9f96da509c72e78bccb5dbf8afe2a7cadd
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

    indexer
        .concurrent_pipeline(BlobPipeline, ConcurrentConfig::default())
        .await?;

    // indexer
    //     .sequential_pipeline(BlobPipeline, SequentialConfig::default())
    //     .await?;

    // indexer
    //     .concurrent_pipeline(BlobIdPipeline, ConcurrentConfig::default())
    //     .await?;

    println!("Starting indexer...");
    let _ = indexer.run().await?.await;
    
    Ok(())
}
