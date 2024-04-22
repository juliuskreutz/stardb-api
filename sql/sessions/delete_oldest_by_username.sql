DELETE FROM sessions
WHERE uuid IN (
        SELECT
            uuid
        FROM
            sessions
        WHERE
            username = $1
        ORDER BY
            expiry DESC OFFSET 9);

