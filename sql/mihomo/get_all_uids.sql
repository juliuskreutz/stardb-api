SELECT
    uid
FROM
    mihomo
WHERE
    NOT EXISTS (
        SELECT
            *
        FROM
            connections
        WHERE
            connections.uid = mihomo.uid
            AND private);

