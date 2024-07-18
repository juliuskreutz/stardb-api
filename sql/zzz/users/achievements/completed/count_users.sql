SELECT
    count(*)
FROM (
    SELECT
        username
    FROM
        zzz_users_achievements_completed
    GROUP BY
        username
    HAVING
        count(*) >= $1) t;

