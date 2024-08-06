DELETE FROM gi_users_achievements_favorites
WHERE username = $1
    AND id = $2;

