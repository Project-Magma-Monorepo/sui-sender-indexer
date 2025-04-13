use tokio::sync::oneshot;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use sui_types::full_checkpoint_content::CheckpointData;
use sui_data_ingestion_core::{
    Worker, WorkerPool, ReaderOptions,
    DataIngestionMetrics, FileProgressStore,
    IndexerExecutor
};
use std::path::PathBuf;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

// Import your existing pipelines
use crate::pipelines::{BlobPipeline, BlobIdPipeline, SenderPipeline};
use crate::db;

// Implement the checkpoint reader
#[derive(Clone)]
pub struct LocalCheckpointReader {
    checkpoint_dir: PathBuf,
    db_pool: db::Pool,
}

impl LocalCheckpointReader {
    pub fn new(checkpoint_dir: PathBuf, db_pool: db::Pool) -> Self {
        Self { 
            checkpoint_dir,
            db_pool
        }
    }
}

#[async_trait]
impl Worker for LocalCheckpointReader {
    type Result = ();

    async fn process_checkpoint(&self, checkpoint: &CheckpointData) -> Result<()> {
        // Log the checkpoint being processed
        println!("Processing checkpoint: {}", checkpoint.checkpoint_summary.sequence_number);
        
        // Create an Arc to share the checkpoint data
        let checkpoint_arc = Arc::new(checkpoint.clone());
        
        // Get a database connection from the pool
        let mut conn = self.db_pool.get().await?;
        
        // Process with BlobPipeline
        let blob_pipeline = BlobPipeline;
        let blob_values = blob_pipeline.process(&checkpoint_arc)?;
        if !blob_values.is_empty() {
            println!("Inserting {} blob records", blob_values.len());
            BlobPipeline::commit(&blob_values, &mut conn).await?;
        }
        
        // Process with BlobIdPipeline
        let blob_id_pipeline = BlobIdPipeline;
        let blob_id_values = blob_id_pipeline.process(&checkpoint_arc)?;
        if !blob_id_values.is_empty() {
            println!("Inserting {} blob ID records", blob_id_values.len());
            BlobIdPipeline::commit(&blob_id_values, &mut conn).await?;
        }
        
        // Process with SenderPipeline
        let sender_pipeline = SenderPipeline;
        let sender_values = sender_pipeline.process(&checkpoint_arc)?;
        if !sender_values.is_empty() {
            println!("Inserting {} sender records", sender_values.len());
            SenderPipeline::commit(&sender_values, &mut conn).await?;
        }
        
        println!("Successfully processed checkpoint: {}", checkpoint.checkpoint_summary.sequence_number);
        Ok(())
    }
}

// Main executor setup
pub async fn run_local_indexer(
    checkpoint_dir: PathBuf,
    db_url: &str,
    concurrency: usize,
    progress_file: PathBuf,
) -> Result<()> {
    // Create database pool
    let db_pool = db::create_pool(db_url).await?;
    
    let (_, exit_receiver) = oneshot::channel();
    let metrics = DataIngestionMetrics::new(&prometheus::Registry::new());
    
    let progress_store = FileProgressStore::new(progress_file);
    let mut executor = IndexerExecutor::new(
        progress_store,
        1, // number of workflow types
        metrics,
    );

    let worker_pool = WorkerPool::new(
        LocalCheckpointReader::new(checkpoint_dir.clone(), db_pool),
        "local_checkpoint_reader".to_string(),
        concurrency,
    );

    executor.register(worker_pool).await?;

    executor.run(
        checkpoint_dir,
        None, // No remote options needed for local reading
        vec![], // No additional options needed
        ReaderOptions::default(),
        exit_receiver,
    ).await?;

    Ok(())
}