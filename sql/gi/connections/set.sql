INSERT INTO gi_connections (uid, username, verified, private)
    VALUES ($1, $2, $3, $4)
ON CONFLICT (uid, username)
    DO UPDATE SET
        verified = EXCLUDED.verified;

