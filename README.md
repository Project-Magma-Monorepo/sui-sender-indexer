
# Sui Sender Indexer

This indexer is designed to index blob objects from the Sui blockchain and store them in a database.

## Features

- Indexes blob objects from Sui checkpoints
- Stores blob metadata in a PostgreSQL database
- Provides a simple API to query blob data
- Efficient storage of blob IDs for quick lookups and reduced storage requirements

## Recent Updates

The indexer has been updated to use a streamlined database schema:

- The `blobs` table now has a simpler structure that aligns with the Sui BlobData model
- Removed redundant fields and added appropriate indexes for better performance
- The `blob_ids` table provides a lightweight alternative for applications that only need object IDs

## Prerequisites

- Rust toolchain
- PostgreSQL database
- Docker (optional, for containerized deployment)

## Setup

1. Clone the repository
2. Set up the database:
   ```bash
   docker-compose up -d db
   ```
3. Run migrations:
   ```bash
   diesel migration run
   ```
4. Build the indexer:
   ```bash
   cargo build
   ```

## Database Schema

### Blobs Table
The `blobs` table stores detailed information about blobs:
- `id`: The object ID (primary key)
- `registered_epoch`: When the blob was registered
- `certified_epoch`: When the blob was certified (optional)
- `deletable`: Whether the blob can be deleted
- `encoding_type`: The encoding format
- `size`: The size of the blob in bytes
- `storage_id`: The ID of the storage object

### BlobIds Table
The `blob_ids` table contains only the essential blob ID for lightweight lookups.

## Testing the Blob Indexer

The simplest way to test the blob indexing functionality is to use the `simple_blob_check` test:

```bash
# Test the latest checkpoint
cargo run --bin simple_blob_check

# Test a specific checkpoint
cargo run --bin simple_blob_check -- --checkpoint_number 12345

# Sui Sender Index

[![Check latest dependencies](https://github.com/amnn/sui-sender-indexer/actions/workflows/deps.yml/badge.svg)](https://github.com/amnn/sui-sender-indexer/actions/workflows/deps.yml)

A demo using `sui-indexer-alt-framework` to index all active addresses since
genesis from the chain. To set things up, run:

```sh
$ diesel setup                                                                \
    --database-url="postgres://postgres:postgrespw@localhost:5432/sui_sender" \
    --migration-dir migrations
$ diesel migration run                                                        \
    --database-url="postgres://postgres:postgrespw@localhost:5432/sui_sender" \
    --migration-dir migrations
```

This test will:
1. Fetch a checkpoint (latest or specified)
2. Check for blobs in the checkpoint
3. Display information about any blobs found

### Testing the BlobId Pipeline

To test the BlobId pipeline specifically:

```bash
# Test if blobs can be detected in a specific checkpoint
cargo run --bin test_blob_id_pipeline -- --checkpoint_number 153130834

# Query the blob_ids table from the database
cargo run --bin query_blob_ids
```

The `test_blob_id_pipeline` test will output information about blob objects found in the checkpoint, and the `query_blob_ids` script will display the stored blob IDs from the database.

### Quick Testing with Docker

You can run the test in a Docker container:

```bash
# Build the test container
docker build -f Dockerfile.test -t sui-blob-indexer-test .

# Run the test
docker run sui-blob-indexer-test simple_blob_check
```

For more detailed testing information, see the [tests/README.md](tests/README.md) file.

## Running the Indexer

To run the full indexer:

```bash
cargo run --bin sui-sender-indexer
```

### Available Pipelines

The indexer includes several pipelines:

1. **SenderPipeline**: Indexes the sender addresses of transactions
2. **BlobPipeline**: Indexes full blob metadata including size, encoding type, etc.
3. **BlobIdPipeline**: Indexes only the IDs of blob objects for efficient storage and quick lookups

You can configure which pipelines are active by modifying the main.rs file.

## Configuration

The indexer can be configured using environment variables:

- `DATABASE_URL`: PostgreSQL connection string (default: postgres://postgres:sui-indexer@localhost:5432/sui_sender)
- `REMOTE_STORE_URL`: URL of the checkpoint store (default: https://checkpoints.testnet.sui.io)
- `START_CHECKPOINT`: Checkpoint number to start indexing from (default: latest)

## Docker Deployment

To deploy the indexer using Docker:

```bash
docker-compose up -d
```

This will start both the database and the indexer.

