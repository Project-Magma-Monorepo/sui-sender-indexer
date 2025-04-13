use std::sync::Arc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use sui_indexer_alt_framework::{
    db,
    pipeline::{sequential::Handler, Processor},
    types::full_checkpoint_content::CheckpointData,
    FieldCount, Result,
};
use sui_types::{base_types::{ObjectID, SuiAddress}, object::Object};

use crate::schema::{kiosks, kiosk_owner_caps};

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = kiosks)]
pub struct StoredKiosk {
    pub id: Vec<u8>,
    pub owner: Vec<u8>,
    pub profits: i64,
    pub item_count: i32,
    pub allow_extensions: bool,
}

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = kiosk_owner_caps)]
pub struct StoredKioskOwnerCap {
    pub id: Vec<u8>,
    pub for_kiosk: Vec<u8>,
    pub current_owner: Vec<u8>,
}

#[derive(Deserialize, Debug)]
pub struct KioskData {
    id: ObjectID,
    profits: Balance,
    owner: ObjectID,
    item_count: u32,
    allow_extensions: bool
}

#[derive(Deserialize, Debug)]
pub struct Balance {
    value: u64,
}

#[derive(Deserialize, Debug)]
pub struct KioskOwnerCapData {
    id: ObjectID,
    for_kiosk: ObjectID
}

// Helper functions to identify object types
fn is_kiosk(obj: &Object) -> bool {
    if let Some(type_) = obj.type_() {
        type_.module().as_str() == "kiosk" && 
        type_.name().as_str() == "Kiosk"
    } else {
        false
    }
}

fn is_kiosk_owner_cap(obj: &Object) -> bool {
    if let Some(type_) = obj.type_() {
        type_.module().as_str() == "kiosk" && 
        type_.name().as_str() == "KioskOwnerCap"
    } else {
        false
    }
}

// Kiosk Pipeline
pub struct KioskPipeline;

impl Processor for KioskPipeline {
    const NAME: &'static str = "kiosks";
    type Value = StoredKiosk;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let kiosks = checkpoint
            .transactions
            .iter()
            .flat_map(|tx| tx.output_objects.iter())
            .filter_map(|obj| {
                if is_kiosk(obj) {
                    obj.data.try_as_move().and_then(|_| 
                        obj.to_rust::<KioskData>().map(|data| StoredKiosk {
                            id: obj.id().to_vec(),
                            owner: data.owner.to_vec(),
                            profits: data.profits.value as i64,
                            item_count: data.item_count as i32,
                            allow_extensions: data.allow_extensions,
                        })
                    )
                } else {
                    None
                }
            })
            .collect();

        Ok(kiosks)
    }
}

#[async_trait::async_trait]
impl Handler for KioskPipeline {
    const MIN_EAGER_ROWS: usize = 50;
    const MAX_BATCH_CHECKPOINTS: usize = 5 * 60;
    type Batch = Vec<StoredKiosk>;

    fn batch(batch: &mut Self::Batch, values: Vec<Self::Value>) {
        batch.extend(values);
    }

    async fn commit(batch: &Self::Batch, conn: &mut db::Connection<'_>) -> Result<usize> {
        if batch.is_empty() {
            return Ok(0);
        }

        diesel::insert_into(kiosks::table)
            .values(batch)
            .on_conflict(kiosks::id)
            .do_update()
            .set((
                kiosks::owner.eq(diesel::dsl::sql("EXCLUDED.owner")),
                kiosks::item_count.eq(diesel::dsl::sql("EXCLUDED.item_count")),
                kiosks::profits.eq(diesel::dsl::sql("EXCLUDED.profits")),
                kiosks::allow_extensions.eq(diesel::dsl::sql("EXCLUDED.allow_extensions")),
            ))
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}

// KioskOwnerCap Pipeline
pub struct KioskOwnerCapPipeline;

impl Processor for KioskOwnerCapPipeline {
    const NAME: &'static str = "kiosk_owner_caps";
    type Value = StoredKioskOwnerCap;
    const FANOUT: usize = 1;

    fn process(&self, checkpoint: &Arc<CheckpointData>) -> Result<Vec<Self::Value>> {
        let owner_caps = checkpoint
            .transactions
            .iter()
            .flat_map(|tx| tx.output_objects.iter())
            .filter_map(|obj| {
                if is_kiosk_owner_cap(obj) {
                    obj.data.try_as_move().and_then(|_| 
                        obj.to_rust::<KioskOwnerCapData>().map(|data| StoredKioskOwnerCap {
                            id: obj.id().to_vec(),
                            for_kiosk: data.for_kiosk.to_vec(),
                            current_owner: obj.get_single_owner()
                                .get_or_insert(SuiAddress::default())
                                .to_vec(),
                        })
                    )
                } else {
                    None
                }
            })
            .collect();

        Ok(owner_caps)
    }
}

#[async_trait::async_trait]
impl Handler for KioskOwnerCapPipeline {
    const MIN_EAGER_ROWS: usize = 50;
    const MAX_BATCH_CHECKPOINTS: usize = 5 * 60;
    type Batch = Vec<StoredKioskOwnerCap>;

    fn batch(batch: &mut Self::Batch, values: Vec<Self::Value>) {
        batch.extend(values);
    }

    async fn commit(batch: &Self::Batch, conn: &mut db::Connection<'_>) -> Result<usize> {
        if batch.is_empty() {
            return Ok(0);
        }

        diesel::insert_into(kiosk_owner_caps::table)
            .values(batch)
            .on_conflict(kiosk_owner_caps::id)
            .do_update()
            .set((
                kiosk_owner_caps::for_kiosk.eq(diesel::dsl::sql("EXCLUDED.for_kiosk")),
                kiosk_owner_caps::current_owner.eq(diesel::dsl::sql("EXCLUDED.current_owner")),
            ))
            .execute(conn)
            .await
            .map_err(Into::into)
    }
}