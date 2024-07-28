SELECT
    id
FROM
    gi_achievements
WHERE
    NOT (hidden
        AND impossible);

