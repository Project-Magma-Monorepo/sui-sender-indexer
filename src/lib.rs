use std::sync::Arc;

use anyhow::Result;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use sui_field_count::FieldCount;
use sui_indexer_alt_framework::pipeline::{concurrent::Handler, Processor};
use sui_pg_db as db;
use sui_types::full_checkpoint_content::CheckpointData;

use crate::schema::senders;

mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Insertable, Debug, FieldCount)]
#[diesel(table_name = senders)]
pub struct StoredSender {
    sender: Vec<u8>,
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
