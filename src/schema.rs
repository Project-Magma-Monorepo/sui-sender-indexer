diesel::table! {
    blob_ids (id) {
        id -> Bytea,
    }
}

diesel::table! {
    blobs (id) {
        id -> Bytea,
        blob_id -> Varchar,
        registered_epoch -> Int8,
        certified_epoch -> Nullable<Int8>,
        deletable -> Bool,
        encoding_type -> Int4,
        size -> Varchar,
        storage_id -> Bytea,
        storage_start_epoch -> Int8,
        storage_end_epoch -> Int8,
        storage_size -> VarChar,
    }
}

diesel::table! {
    senders (sender) {
        sender -> Bytea,
    }
}