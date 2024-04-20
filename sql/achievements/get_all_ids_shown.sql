SELECT
    id
FROM
    achievements
WHERE
    NOT (hidden
        AND impossible);

