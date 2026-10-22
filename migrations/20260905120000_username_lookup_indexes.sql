-- Session pruning and account connection lists filter by username. Existing
-- UUID and (uid, username) primary keys do not provide that leading access path.
-- These ordinary index builds run atomically; schedule for low traffic on large tables.
CREATE INDEX sessions_username_idx ON sessions (username);
CREATE INDEX connections_username_idx ON connections (username);
CREATE INDEX gi_connections_username_idx ON gi_connections (username);
CREATE INDEX zzz_connections_username_idx ON zzz_connections (username);
