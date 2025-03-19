use std::sync::Arc;


use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use sui_indexer_alt_framework::db;
use sui_indexer_alt_framework::pipeline::{concurrent::Handler, Processor};
use sui_indexer_alt_framework::types::full_checkpoint_content::CheckpointData;
use sui_indexer_alt_framework::FieldCount;
use sui_indexer_alt_framework::Result;
use sui_types::base_types::ObjectID;
use sui_types::object::Object;


use serde::{Deserialize, Serialize};



use crate::schema::{blobs, senders, blob_ids};

pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// The Blob module ID
const BLOB_MODULE_ADDRESS: &str = "0x795ddbc26b8cfff2551f45e198b87fc19473f2df50f995376b924ac80e56f88b";
// const BLOB_TYPE: &str =  "0x11f5d87dab9494ce459299c7874e959ff121649fd2d4529965f6dea85c153d2d::blob::Blob";

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = senders)]
pub struct StoredSender {
    sender: Vec<u8>,
}

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = blobs)]
pub struct StoredBlob {
    pub id: Vec<u8>,
    pub blob_id: String,
    pub registered_epoch: i64,
    pub certified_epoch: Option<i64>,
    pub deletable: bool,
    pub encoding_type: i32,
    pub size: String,
    pub storage_id: Vec<u8>,
    pub storage_start_epoch: i64,
    pub storage_end_epoch: i64,
    pub storage_size: String,
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
        // Use streaming operations instead of for loops
        Ok(checkpoint
            .transactions
            .iter()
            .flat_map(|tx| tx.output_objects.iter())
            .filter_map(|obj| extract_blob_data(obj))
            .collect())
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
pub fn is_blob_object(obj: &Object) -> bool {
    if let Some(move_object_type) = obj.data.type_() {
        // Sui blobs are in the storage module, with struct name Blob
        let is_blob = move_object_type.module().as_str() == "blob" && 
                      move_object_type.name().as_str() == "Blob";
        
        if is_blob {
            println!("Found Blob with ID: {}, {:?}", obj.id(), obj.data);
            return true;
        }
        return false;
    }
    false
}

// Helper function to extract Blob data from an object
pub fn extract_blob_data(obj: &Object) -> Option<StoredBlob> {
    println!("Checking if object is a Blob");

    if !is_blob_object(obj) {
        return None;
    }

    // Get the Move object
    let move_obj = obj.data.try_as_move()?;
    println!("Attempting to deserialize blob object with ID: {}", obj.id());
    println!("Content length: {} bytes", move_obj.contents().len());
    
    // Define structs for BCS deserialization
    #[derive(Deserialize)]
    struct BlobData {
        id: ObjectID, // Every Move object has an ID field
        blob_id: String,
        registered_epoch: u64,
        certified_epoch: Option<u64>,
        deletable: bool,
        encoding_type: u64,
        size: String,
        storage: StorageData,
    }

    #[derive(Deserialize)]
    struct StorageData {
        id: StorageID,
        start_epoch: u64,
        end_epoch: u64,
        storage_size: String,
    }

    #[derive(Deserialize)]
    struct StorageID {
        id: ObjectID,
    }
    
    // Try to deserialize with error handling
    let result = bcs::from_bytes::<BlobData>(move_obj.contents());
    
    match result {
        Ok(blob_data) => {
            println!("Successfully deserialized blob with blob_id: {}", blob_data.blob_id);
            
            Some(StoredBlob {
                id: obj.id().to_vec(),
                blob_id: blob_data.blob_id,
                registered_epoch: blob_data.registered_epoch as i64,
                certified_epoch: blob_data.certified_epoch.map(|e| e as i64),
                deletable: blob_data.deletable,
                encoding_type: blob_data.encoding_type as i32,
                size: blob_data.size,
                storage_id: blob_data.storage.id.id.to_vec(),
                storage_start_epoch: blob_data.storage.start_epoch as i64,
                storage_end_epoch: blob_data.storage.end_epoch as i64,
                storage_size: blob_data.storage.storage_size,
            })
        },
        Err(e) => {
            println!("Failed to deserialize blob: {}", e);
            None
        }
    }
}

// Simple struct to hold just blob ID information
#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = blob_ids)]
pub struct StoredBlobId {
    pub id: Vec<u8>,
}

pub struct BlobIdPipeline;

impl Processor for BlobIdPipeline {
    const NAME: &'static str = "blob_ids";

    type Value = StoredBlobId;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let blobs = checkpoint
            .transactions
            .iter()
            .flat_map(|tx| tx.output_objects.iter())
            .filter(|obj| is_blob_object(obj))
            .map(|obj| {
                println!("Found blob with object ID: {}", obj.id());
                StoredBlobId {
                    id: obj.id().to_vec(),
                }
            })
            .collect();
        
        Ok(blobs)
    }
}

#[async_trait::async_trait]
impl Handler for BlobIdPipeline {
    async fn commit(values: &[Self::Value], conn: &mut db::Connection<'_>) -> Result<usize> {
        // Only insert the minimal IDs to the database
        diesel::insert_into(blob_ids::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}
