// @generated automatically by Diesel CLI.

diesel::table! {
    senders (sender) {
        sender -> Bytea,
    }
}

diesel::table! {
    blob_objects (id) {
        id -> Bytea,
        blob_id -> Text,
        registered_epoch -> Int8,
        certified_epoch -> Nullable<Int8>,
        deletable -> Bool,
        encoding_type -> Int4,
        size -> Text,
        storage_id -> Bytea,
        storage_start_epoch -> Int8,
        storage_end_epoch -> Int8,
        storage_size -> Text,
    }
}
