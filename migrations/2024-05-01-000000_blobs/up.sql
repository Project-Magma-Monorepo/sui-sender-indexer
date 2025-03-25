CREATE TABLE IF NOT EXISTS blobs (
    id BYTEA PRIMARY KEY,
    registered_epoch BIGINT NOT NULL,
    certified_epoch BIGINT,
    deletable BOOLEAN NOT NULL,
    encoding_type INTEGER NOT NULL,
    size VARCHAR NOT NULL,
    blob_id BYTEA,
    storage_id BYTEA NOT NULL,
    storage_start_epoch BIGINT,
    storage_end_epoch BIGINT,
    storage_size BIGINT
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS blobs_registered_epoch_idx ON blobs(registered_epoch);
CREATE INDEX IF NOT EXISTS blobs_storage_id_idx ON blobs(storage_id);
CREATE INDEX IF NOT EXISTS blobs_blob_id_idx ON blobs(blob_id);

CREATE INDEX IF NOT EXISTS blobs_blob_id_idx ON blobs(blob_id);
