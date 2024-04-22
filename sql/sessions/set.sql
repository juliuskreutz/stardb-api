INSERT INTO sessions (uuid, username, expiry)
    VALUES ($1, $2, $3)
ON CONFLICT (uuid)
    DO UPDATE SET
        username = EXCLUDED.username, expiry = EXCLUDED.expiry;

