CREATE TABLE IF NOT EXISTS light_cones_text (
    id INT4 NOT NULL REFERENCES light_cones ON DELETE CASCADE,
    language TEXT NOT NULL,
    name TEXT NOT NULL,
    PRIMARY KEY(id, language)
);