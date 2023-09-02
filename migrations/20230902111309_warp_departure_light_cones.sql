CREATE TABLE IF NOT EXISTS warp_departure_light_cones (
    id INT8 NOT NULL,
    uid INT8 NOT NULL REFERENCES mihomo ON DELETE CASCADE,
    light_cone INT4 NOT NULL REFERENCES light_cones ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL,
    PRIMARY KEY(id, uid)
);