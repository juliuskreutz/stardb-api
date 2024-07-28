WITH threshholded_gi_users_achievements AS (
    SELECT
        gi_users_achievements_completed.username,
        id
    FROM
        gi_users_achievements_completed
        JOIN (
            SELECT
                username
            FROM
                gi_users_achievements_completed
            GROUP BY
                username
            HAVING
                count(*) >= $1) threshholded_users ON gi_users_achievements_completed.username = threshholded_users.username
),
gi_achievements_percent AS (
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
                        threshholded_gi_users_achievements
                    WHERE
                        users.username = threshholded_gi_users_achievements.username)) percent
            FROM
                threshholded_gi_users_achievements
            GROUP BY
                id)
    INSERT INTO gi_achievements_percent (id, percent)
SELECT
    gi_achievements.id,
    COALESCE(percent, 0)
FROM
    gi_achievements
    LEFT JOIN gi_achievements_percent ON gi_achievements.id = gi_achievements_percent.id
ON CONFLICT (id)
    DO UPDATE SET
        percent = EXCLUDED.percent;

