INSERT INTO gi_profiles (uid, name)
    VALUES ($1, $2)
ON CONFLICT (uid)
    DO UPDATE SET
        name = EXCLUDED.name;

