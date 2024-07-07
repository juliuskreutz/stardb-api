WITH threshholded_zzz_users_achievements AS (
    SELECT
        zzz_users_achievements_completed.username,
        id
    FROM
        zzz_users_achievements_completed
        JOIN (
            SELECT
                username
            FROM
                zzz_users_achievements_completed
            GROUP BY
                username
            HAVING
                count(*) >= $1) threshholded_users ON zzz_users_achievements_completed.username = threshholded_users.username
),
zzz_achievements_percent AS (
    SELECT
        id,
        COUNT(*)::float / (
            SELECT
                COUNT(*)
            FROM
                users
            WHERE
                EXISTS (
                    SELECT
                        *
                    FROM
                        threshholded_zzz_users_achievements
                    WHERE
                        users.username = threshholded_zzz_users_achievements.username)) percent
            FROM
                threshholded_zzz_users_achievements
            GROUP BY
                id)
    INSERT INTO zzz_achievements_percent (id, percent)
SELECT
    zzz_achievements.id,
    COALESCE(percent, 0)
FROM
    zzz_achievements
    LEFT JOIN zzz_achievements_percent ON zzz_achievements.id = zzz_achievements_percent.id
ON CONFLICT (id)
    DO UPDATE SET
        percent = EXCLUDED.percent;

