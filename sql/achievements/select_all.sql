INSERT INTO users_achievements_completed (username, id)
SELECT
    $1,
    id
FROM
    achievements
WHERE
    SET IS NULL AND NOT impossible
ON CONFLICT (username, id)
    DO NOTHING;

