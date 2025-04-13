CREATE TABLE kiosk_owner_caps (
    id BYTEA PRIMARY KEY,
    for_kiosk BYTEA NOT NULL,
    current_owner BYTEA NOT NULL,
);

CREATE TABLE kiosks (
    id BYTEA PRIMARY KEY,
    profits BIGINT NOT NULL,
    owner BYTEA NOT NULL,
    item_count INTEGER NOT NULL,
    allow_extensions BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE nfts (
    id BYTEA PRIMARY KEY,
    kiosk_id BYTEA REFERENCES kiosks(id),
);

CREATE INDEX kiosk_owner_caps_for_idx ON kiosk_owner_caps(for_kiosk);
CREATE INDEX kiosks_owner_idx ON kiosks(owner);
CREATE INDEX nfts_kiosk_idx ON nfts(kiosk_id); 