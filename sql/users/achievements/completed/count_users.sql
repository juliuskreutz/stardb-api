SELECT
    count(*)
FROM (
    SELECT
        username
    FROM
        users_achievements_completed
    GROUP BY
        username
    HAVING
        count(*) >= $1) t;

