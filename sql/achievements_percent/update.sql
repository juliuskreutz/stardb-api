WITH threshholded_users_achievements AS (
    SELECT
        users_achievements_completed.username,
        id
    FROM
        users_achievements_completed
        JOIN (
            SELECT
                username
            FROM
                users_achievements_completed
            GROUP BY
                username
            HAVING
                count(*) >= $1) threshholded_users ON users_achievements_completed.username = threshholded_users.username
),
achievements_percent AS (
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
                        threshholded_users_achievements
                    WHERE
                        users.username = threshholded_users_achievements.username)) percent
            FROM
                threshholded_users_achievements
            GROUP BY
                id)
    INSERT INTO achievements_percent (id, percent)
SELECT
    achievements.id,
    COALESCE(percent, 0)
FROM
    achievements
    LEFT JOIN achievements_percent ON achievements.id = achievements_percent.id;

