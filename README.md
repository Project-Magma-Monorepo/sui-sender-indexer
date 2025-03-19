# Sui Sender Indexer

This indexer is designed to index blob objects from the Sui blockchain and store them in a database.

## Features

- Indexes blob objects from Sui checkpoints
- Stores blob metadata in a PostgreSQL database
- Provides a simple API to query blob data
- Efficient storage of blob IDs for quick lookups and reduced storage requirements

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
3. Build the indexer:
   ```bash
   cargo build
   ```

## Testing the Blob Indexer

The simplest way to test the blob indexing functionality is to use the `simple_blob_check` test:

```bash
# Test the latest checkpoint
cargo run --bin simple_blob_check

# Test a specific checkpoint
cargo run --bin simple_blob_check -- --checkpoint_number 12345
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

- `DATABASE_URL`: PostgreSQL connection string (default: postgres://supabase_admin:sui-indexer@localhost:5432/sui_sender)
- `REMOTE_STORE_URL`: URL of the checkpoint store (default: https://checkpoints.testnet.sui.io)
- `START_CHECKPOINT`: Checkpoint number to start indexing from (default: latest)

## Docker Deployment

To deploy the indexer using Docker:

```bash
docker-compose up -d
```

This will start both the database and the indexer.

