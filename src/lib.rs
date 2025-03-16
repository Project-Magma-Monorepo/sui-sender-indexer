use std::sync::Arc;
use std::str::FromStr;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use serde_json::Value;
use sui_indexer_alt_framework::db;
use sui_indexer_alt_framework::pipeline::{concurrent::Handler, Processor};
use sui_indexer_alt_framework::types::full_checkpoint_content::CheckpointData;
use sui_indexer_alt_framework::FieldCount;
use sui_indexer_alt_framework::Result;
use sui_types::base_types::ObjectID;
use sui_types::object::Object;

use crate::schema::{blobs, senders};

mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// The Blob module ID
const BLOB_MODULE_ID: &str = "0x795ddbc26b8cfff2551f45e198b87fc19473f2df50f995376b924ac80e56f88b::blob::Blob";

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = senders)]
pub struct StoredSender {
    sender: Vec<u8>,
}

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = blobs)]
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

pub struct BlobPipeline;

impl Processor for BlobPipeline {
    const NAME: &'static str = "blobs";

    type Value = StoredBlob;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let mut blobs = Vec::new();

        // Iterate through all transactions in the checkpoint
        for tx in &checkpoint.transactions {
            // Check all output objects for Blob type
            for obj in &tx.output_objects {
                if is_blob_object(obj) {
                    if let Some(blob) = extract_blob_data(obj) {
                        blobs.push(blob);
                    }
                }
            }
        }

        Ok(blobs)
    }
}

#[async_trait::async_trait]
impl Handler for BlobPipeline {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        diesel::insert_into(blobs::table)
            .values(values)
            .on_conflict(blobs::id)
            .do_update()
            .set((
                blobs::blob_id.eq(diesel::dsl::sql("EXCLUDED.blob_id")),
                blobs::registered_epoch.eq(diesel::dsl::sql("EXCLUDED.registered_epoch")),
                blobs::certified_epoch.eq(diesel::dsl::sql("EXCLUDED.certified_epoch")),
                blobs::deletable.eq(diesel::dsl::sql("EXCLUDED.deletable")),
                blobs::encoding_type.eq(diesel::dsl::sql("EXCLUDED.encoding_type")),
                blobs::size.eq(diesel::dsl::sql("EXCLUDED.size")),
                blobs::storage_id.eq(diesel::dsl::sql("EXCLUDED.storage_id")),
                blobs::storage_start_epoch.eq(diesel::dsl::sql("EXCLUDED.storage_start_epoch")),
                blobs::storage_end_epoch.eq(diesel::dsl::sql("EXCLUDED.storage_end_epoch")),
                blobs::storage_size.eq(diesel::dsl::sql("EXCLUDED.storage_size")),
            ))
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}

// Helper function to check if an object is a Blob
fn is_blob_object(obj: &Object) -> bool {
    if let Some(type_str) = obj.type_().map(|t| t.to_string()) {
        type_str == BLOB_MODULE_ID
    } else {
        false
    }
}

// Helper function to extract Blob data from an object
fn extract_blob_data(obj: &Object) -> Option<StoredBlob> {
    // Convert the object to JSON for easier field access
    let json_obj = serde_json::to_value(obj).ok()?;
    
    // Access the content field which contains the Move object data
    let content = json_obj.get("content")?.get("fields")?;
    
    // Extract blob_id
    let blob_id = content.get("blob_id")?.as_str()?.to_string();
    
    // Extract registered_epoch
    let registered_epoch = content.get("registered_epoch")?.as_u64()? as i64;
    
    // Extract certified_epoch (optional)
    let certified_epoch = if content.get("certified_epoch")?.is_null() {
        None
    } else {
        Some(content.get("certified_epoch")?.as_u64()? as i64)
    };
    
    // Extract deletable
    let deletable = content.get("deletable")?.as_bool()?;
    
    // Extract encoding_type
    let encoding_type = content.get("encoding_type")?.as_u64()? as i32;
    
    // Extract size
    let size = content.get("size")?.as_str()?.to_string();
    
    // Extract storage fields
    let storage = content.get("storage")?.get("fields")?;
    
    // Extract storage ID
    let storage_id_str = storage.get("id")?.get("fields")?.get("id")?.as_str()?;
    let storage_id = ObjectID::from_str(storage_id_str).ok()?.to_vec();
    
    // Extract storage epochs
    let start_epoch = storage.get("start_epoch")?.as_u64()? as i64;
    let end_epoch = storage.get("end_epoch")?.as_u64()? as i64;
    
    // Extract storage size
    let storage_size = storage.get("storage_size")?.as_str()?.to_string();
    
    Some(StoredBlob {
        id: obj.id().to_vec(),
        blob_id,
        registered_epoch,
        certified_epoch,
        deletable,
        encoding_type,
        size,
        storage_id,
        storage_start_epoch: start_epoch,
        storage_end_epoch: end_epoch,
        storage_size,
    })
}
