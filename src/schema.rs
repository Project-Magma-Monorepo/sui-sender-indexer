// @generated automatically by Diesel CLI.

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

diesel::allow_tables_to_appear_in_same_query!(
    blobs,
    senders,
);
