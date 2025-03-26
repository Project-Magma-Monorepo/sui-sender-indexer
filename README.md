# Sui Sender Index

[![Check latest dependencies](https://github.com/amnn/sui-sender-indexer/actions/workflows/deps.yml/badge.svg)](https://github.com/amnn/sui-sender-indexer/actions/workflows/deps.yml)

A demo using `sui-indexer-alt-framework` to index all active addresses since
genesis from the chain. To set things up, run:

Push things to repo
GIT_SSH_COMMAND="ssh -i ~/MasterProject/SshKeys/sshKeyLoris" git push

Start Postgre db : 
docker compose up -d postgres

Connect to docker CLI DB:
docker compose exec postgres bash

Start Postgres:
sudo /etc/init.d/postgresql start

Launch Postgres CLI:
sudo -u postgres psql;

Launch From docker : 
psql -U postgres -d sui_sender

List tables on DB:
\l

Access DB once inside CLI:
\connect sui_sender;

List all relations:
\dt

Access a relation:
SELECT * FROM senders;

DB PW : 
sui-indexer


```sh
$ diesel setup                                                                \
    --database-url="postgres://postgres:sui-indexer@localhost:5432/sui_sender" \
    --migration-dir migrations
$ diesel migration run                                                        \
    --database-url="postgres://postgres:sui-indexer@localhost:5432/sui_sender" \
    --migration-dir migrations
```

To run the indexer:

```sh
$ RUST_LOG=info cargo run --release -- \
    --remote-store-url https://checkpoints.mainnet.sui.io
```

(The indexer defaults to populating the database set-up in the previous code
snippet, but this can be overridden using the `--database-url` flag).

To Run Alex for blob

```sh
$ docker compose build (optional flag --no-cache)
$ docker compose up 
```

build avant de run 