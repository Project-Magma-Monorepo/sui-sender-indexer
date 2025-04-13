FROM rust:latest as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev clang llvm-dev libclang-dev \
    libzstd-dev liblz4-dev libsnappy-dev libbz2-dev zlib1g-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install diesel_cli
RUN cargo install diesel_cli --no-default-features --features postgres --locked

# Setup app
WORKDIR /app
COPY . .

# Set environment variables for bindgen
ENV LIBCLANG_PATH="/usr/lib/llvm-14/lib"
ENV BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/llvm-14/include"

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 libssl3 ca-certificates libzstd1 liblz4-1 \
    libsnappy1v5 libbz2-1.0 zlib1g \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary - MAKE SURE THE PATH IS CORRECT
COPY --from=builder /app/target/release/sui-sender-indexer /app/

# Copy diesel CLI and migrations
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations/

# Create a single entrypoint script
RUN echo '#!/bin/bash\n\
set -e\n\
echo "Running migrations..."\n\
diesel setup --database-url="$DATABASE_URL" --migration-dir migrations\n\
diesel migration run --database-url="$DATABASE_URL" --migration-dir migrations\n\
echo "Starting indexer..."\n\
exec /app/sui-sender-indexer --remote-store-url "$REMOTE_STORE_URL" --database-url "$DATABASE_URL" --first-checkpoint "$START_CHECKPOINT" --skip-watermark\n\
' > /app/entrypoint.sh && chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]