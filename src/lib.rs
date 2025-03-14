use std::sync::Arc;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use sui_indexer_alt_framework::db;
use sui_indexer_alt_framework::pipeline::{concurrent::Handler, Processor};
use sui_indexer_alt_framework::types::full_checkpoint_content::CheckpointData;
use sui_indexer_alt_framework::FieldCount;
use sui_indexer_alt_framework::Result;

use crate::schema::{senders, blob_objects};

mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = senders)]
pub struct StoredSender {
    sender: Vec<u8>,
}

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = blob_objects)]
pub struct StoredBlob {
    id: Vec<u8>,
    blob_id: String,
    registered_epoch: i64,
    certified_epoch: Option<i64>,
    deletable: bool,
    encoding_type: i32,
    size: String,
    storage_id: Vec<u8>,
    storage_start_epoch: i64,
    storage_end_epoch: i64,
    storage_size: String,
}

pub struct SenderPipeline;

impl Processor for SenderPipeline {
    const NAME: &'static str = "senders";

    type Value = StoredSender;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        Ok(checkpoint
            .transactions
            .iter()
            .map(|tx| StoredSender {
                sender: tx.transaction.sender_address().to_vec(),
            })
            .collect())
    }
}

pub struct BlobPipeline;

impl Processor for BlobPipeline {
    const NAME: &'static str = "blob_objects";

    type Value = StoredBlob;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let mut blobs = Vec::new();
        
        for tx in &checkpoint.transactions {
            for effect in &tx.effects.effects {
                for created in &effect.created {
                    // Check if the object is of type Blob
                    if let Some(obj_type) = &created.object_type {
                        if obj_type == "0x795ddbc26b8cfff2551f45e198b87fc19473f2df50f995376b924ac80e56f88b::blob::Blob" {
                            // Fetch the object data
                            if let Some(obj) = checkpoint.objects.get(&created.object_id) {
                                if let Some(content) = &obj.content {
                                    // Parse the object fields from content
                                    if let Ok(blob_data) = serde_json::from_str::<serde_json::Value>(&content.to_string()) {
                                        // Extract the storage object
                                        if let Some(storage) = blob_data.get("storage")
                                            .and_then(|s| s.get("fields")) {
                                            
                                            let storage_id = storage.get("id")
                                                .and_then(|id| id.get("id"))
                                                .and_then(|id| id.as_str())
                                                .map(|id| hex_to_bytes(id.trim_start_matches("0x")))
                                                .unwrap_or_default();
                                                
                                            let storage_start_epoch = storage.get("start_epoch")
                                                .and_then(|e| e.as_i64())
                                                .unwrap_or_default();
                                                
                                            let storage_end_epoch = storage.get("end_epoch")
                                                .and_then(|e| e.as_i64())
                                                .unwrap_or_default();
                                                
                                            let storage_size = storage.get("storage_size")
                                                .and_then(|s| s.as_str())
                                                .unwrap_or_default()
                                                .to_string();
                                            
                                            // Create the StoredBlob
                                            blobs.push(StoredBlob {
                                                id: created.object_id.to_vec(),
                                                blob_id: blob_data.get("blob_id")
                                                    .and_then(|id| id.as_str())
                                                    .unwrap_or_default()
                                                    .to_string(),
                                                registered_epoch: blob_data.get("registered_epoch")
                                                    .and_then(|e| e.as_i64())
                                                    .unwrap_or_default(),
                                                certified_epoch: blob_data.get("certified_epoch")
                                                    .and_then(|e| e.as_i64()),
                                                deletable: blob_data.get("deletable")
                                                    .and_then(|d| d.as_bool())
                                                    .unwrap_or_default(),
                                                encoding_type: blob_data.get("encoding_type")
                                                    .and_then(|e| e.as_i64())
                                                    .map(|e| e as i32)
                                                    .unwrap_or_default(),
                                                size: blob_data.get("size")
                                                    .and_then(|s| s.as_str())
                                                    .unwrap_or_default()
                                                    .to_string(),
                                                storage_id,
                                                storage_start_epoch,
                                                storage_end_epoch,
                                                storage_size,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(blobs)
    }
}

#[async_trait::async_trait]
impl Handler for SenderPipeline {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        diesel::insert_into(senders::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl Handler for BlobPipeline {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        diesel::insert_into(blob_objects::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}

// Helper function to convert hex string to bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| {
            if i + 2 <= hex.len() {
                u8::from_str_radix(&hex[i..i + 2], 16).ok()
            } else {
                None
            }
        })
        .collect()
}
