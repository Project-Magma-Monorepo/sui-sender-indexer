use anyhow::{Context, Result};
use clap::Parser;
use prometheus::Registry;
use sui_indexer_alt_framework::{
    ingestion::{ClientArgs, IngestionConfig},
    pipeline::concurrent::ConcurrentConfig,
    Indexer, IndexerArgs,
};
use sui_indexer_alt_metrics::{MetricsArgs, MetricsService};
use sui_pg_db::DbArgs;
use sui_sender_indexer::{SenderPipeline, MIGRATIONS};
use tokio_util::sync::CancellationToken;
use url::Url;

#[derive(clap::Parser, Debug)]
struct Args {
    #[clap(
        long,
        default_value = "postgres://postgres:sui-indexer@localhost:5432/sui_sender"
    )]
    database_url: Url,

    #[clap(flatten)]
    indexer_args: IndexerArgs,

    #[clap(flatten)]
    client_args: ClientArgs,

    #[clap(flatten)]
    metrics_args: MetricsArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    tracing_subscriber::fmt::init();

    let db_args = DbArgs {
        database_url: args.database_url,
        ..Default::default()
    };

    let cancel = CancellationToken::new();

    let registry = Registry::new_custom(Some("indexer_alt".into()), None)
        .context("Failed to create Prometheus registry.")?;

    let metrics = MetricsService::new(args.metrics_args, registry, cancel.child_token());

    let mut indexer = Indexer::new(
        db_args,
        args.indexer_args,
        args.client_args,
        IngestionConfig::default(),
        &MIGRATIONS,
        metrics.registry(),
        cancel.child_token(),
    )
    .await
    .expect("Failed to create indexer");

    indexer
        .concurrent_pipeline(SenderPipeline, ConcurrentConfig::default())
        .await?;

    let h_metrics = metrics.run().await?;
    let h_indexer = indexer.run().await?;

    let _ = h_indexer.await;
    cancel.cancel();
    let _ = h_metrics.await;
    Ok(())
}
