INSERT INTO gi_users_achievements_completed (username, id)
    VALUES ($1, $2)
ON CONFLICT (username, id)
    DO NOTHING;

