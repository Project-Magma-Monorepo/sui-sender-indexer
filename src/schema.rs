diesel::table! {
    blob_ids (id) {
        id -> Bytea,
    }
}

diesel::table! {
    blobs (id) {
        id -> Bytea,
        registered_epoch -> Int8,
        certified_epoch -> Nullable<Int8>,
        deletable -> Bool,
        encoding_type -> Int4,
        size -> Varchar,
        blob_id -> Bytea,
        storage_id -> Bytea,
        storage_start_epoch -> Int8,
        storage_end_epoch -> Int8,
        storage_size -> Int8,
    }
}

diesel::table! {
    senders (sender) {
        sender -> Bytea,
    }
}