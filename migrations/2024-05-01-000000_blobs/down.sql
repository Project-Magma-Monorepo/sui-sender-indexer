-- Drop the indexes first
DROP INDEX IF EXISTS blobs_registered_epoch_idx;
DROP INDEX IF EXISTS blobs_storage_id_idx;
DROP INDEX IF EXISTS blobs_blob_id_idx;

-- Then drop the table
DROP TABLE IF EXISTS blobs;
