INSERT INTO zzz_uids (uid)
    VALUES ($1)
ON CONFLICT (uid)
    DO NOTHING;

