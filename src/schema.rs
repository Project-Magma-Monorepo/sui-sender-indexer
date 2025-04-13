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

diesel::table! {
    kiosk_owner_caps (id) {
        id -> Bytea,
        for_kiosk -> Bytea,
        current_owner -> Bytea,
    }
}

diesel::table! {
    kiosks (id) {
        id -> Bytea,
        profits -> Int8,
        owner -> Bytea,
        item_count -> Int4,
        allow_extensions -> Bool,
        created_at -> Timestamp,
    }
}

diesel::table! {
    nfts (id) {
        id -> Bytea,
        kiosk_id -> Nullable<Bytea>,
    }
}

diesel::joinable!(nfts -> kiosks (kiosk_id));

diesel::allow_tables_to_appear_in_same_query!(
    blob_ids,
    blobs,
    senders,
    kiosk_owner_caps,
    kiosks,
    nfts,
);