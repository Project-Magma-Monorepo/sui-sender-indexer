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
// Option 1: Use external_crates directly
// use external_crates::move_core_types::u256::U256;

// Import the U256 type from move-core-types
use move_core_types::u256::U256;

use serde::{Deserialize, Serialize};



use crate::schema::{blobs, senders, blob_ids};

pub mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// The Blob module ID
const BLOB_MODULE_ADDRESS: &str = "0x795ddbc26b8cfff2551f45e198b87fc19473f2df50f995376b924ac80e56f88b";
// const BLOB_TYPE: &str =  "0x11f5d87dab9494ce459299c7874e959ff121649fd2d4529965f6dea85c153d2d::blob::Blob";

// Define BlobData structure at the module level so it can be used by multiple functions
#[derive(Deserialize, Debug)]
pub struct BlobData {
    id: ObjectID, // Every Move object has an ID field
    registered_epoch: u32,
    blob_id: U256,  // Fixed from U to U256
    size: u64,
    encoding_type: u8,
    certified_epoch: Option<u32>,
    storage: Storage,
    deletable: bool,
}

#[derive(Deserialize, Debug)]
pub struct Storage {
    id: ObjectID,
    start_epoch: u32,
    end_epoch: u32,
    storage_size: u64,
}



#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = senders)]
pub struct StoredSender {
    sender: Vec<u8>,
}

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = blobs)]
pub struct StoredBlob {
    pub id: Vec<u8>,
    pub registered_epoch: i64,
    pub certified_epoch: Option<i64>,
    pub deletable: bool,
    pub encoding_type: i32,
    pub size: String,
    pub blob_id: Vec<u8>,  // Add the blob_id field as Vec<u8> (BYTEA in PostgreSQL)
    pub storage_id: Vec<u8>,
    pub storage_start_epoch: i64,
    pub storage_end_epoch: i64,
    pub storage_size: i64,
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
                blobs::registered_epoch.eq(diesel::dsl::sql("EXCLUDED.registered_epoch")),
                blobs::certified_epoch.eq(diesel::dsl::sql("EXCLUDED.certified_epoch")),
                blobs::deletable.eq(diesel::dsl::sql("EXCLUDED.deletable")),
                blobs::encoding_type.eq(diesel::dsl::sql("EXCLUDED.encoding_type")),
                blobs::size.eq(diesel::dsl::sql("EXCLUDED.size")),
                blobs::storage_id.eq(diesel::dsl::sql("EXCLUDED.storage_id")),
                blobs::storage_start_epoch.eq(diesel::dsl::sql("EXCLUDED.storage_start_epoch")),
                blobs::storage_end_epoch.eq(diesel::dsl::sql("EXCLUDED.storage_end_epoch")),
                blobs::storage_size.eq(diesel::dsl::sql("EXCLUDED.storage_size")),
                blobs::blob_id.eq(diesel::dsl::sql("EXCLUDED.blob_id")),
            ))
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}

// Helper function to check if an object is a Blob
pub fn is_blob_object(obj: &Object) -> bool {
    if let Some(move_object_type) = obj.data.type_() {
        // Sui blobs are in the blob module, with struct name Blob
        let is_blob = move_object_type.module().as_str() == "blob" && 
                     move_object_type.name().as_str() == "Blob";
        
        if is_blob {
            println!("Found Blob with ID: {}", obj.id());
            return true;
        }
        return false;
    }
    false
}



// Helper function to convert U256 to bytes
fn u256_to_bytes(value: &U256) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

// Helper function to convert bytes back to U256
fn bytes_to_u256(bytes: &[u8]) -> U256 {
    let mut array = [0u8; 32]; // U256 is 32 bytes
    let len = std::cmp::min(bytes.len(), 32);
    array[0..len].copy_from_slice(&bytes[0..len]);
    U256::from_le_bytes(&array)
}

// Helper function to extract Blob data from an object
pub fn extract_blob_data(obj: &Object) -> Option<StoredBlob> {
    if !is_blob_object(obj) {
        return None;
    }

    // Get the Move object
    // Try to deserialize with error handling
    match obj.to_rust::<BlobData>() {
        Some(data) => {
            println!("Successfully deserialized blob data on the whole object: {:?}", data) ;
            
            // Map the BlobData to StoredBlob, ensuring all fields are properly translated
            Some(StoredBlob {
                id: obj.id().to_vec(),
                registered_epoch: data.registered_epoch as i64,
                certified_epoch: data.certified_epoch.map(|e| e as i64),
                deletable: data.deletable,
                encoding_type: data.encoding_type as i32,
                size: data.size.to_string(), // Convert u64 to String
                blob_id: u256_to_bytes(&data.blob_id), // Convert U256 to bytes
                storage_id: data.storage.id.to_vec(),
                storage_start_epoch: data.storage.start_epoch as i64,
                storage_end_epoch: data.storage.end_epoch as i64,
                storage_size: data.storage.storage_size as i64,
            })
        },
        None => {
            println!("Failed to transtypify to Rust: Object couldn't be deserialized to BlobData");
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
            .filter_map(|obj| {
                // Get the Move object to extract the actual blob ID from the BlobData
                let move_obj = obj.data.try_as_move()?;
                    
                // Try to deserialize to get the actual blob ID
                if let Some(blob_data) = move_obj.to_rust::<BlobData>() {
                    println!("Found blob with blob_id and successfully deserialized the whole object: {}", blob_data.blob_id);
                    return Some(StoredBlobId {
                        id: u256_to_bytes(&blob_data.blob_id), // Use the U256 conversion function
                    });
                }
                
                // Fallback to using object ID if deserialization fails
                println!("Using object ID for blob: {}", obj.id());
                Some(StoredBlobId {
                    id: obj.id().to_vec(),
                })
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
