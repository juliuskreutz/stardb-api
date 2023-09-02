CREATE TABLE IF NOT EXISTS warp_departure_characters (
    id INT8 NOT NULL,
    uid INT8 NOT NULL REFERENCES mihomo ON DELETE CASCADE,
    character INT4 NOT NULL REFERENCES characters ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    PRIMARY KEY(id, uid)
);