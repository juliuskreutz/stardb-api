SELECT
    uid
FROM
    zzz_uids
WHERE
    EXISTS (
        SELECT
            *
        FROM
            zzz_signals
        WHERE
            zzz_uids.uid = zzz_signals.uid)
    AND NOT EXISTS (
        SELECT
            *
        FROM
            zzz_connections
        WHERE
            zzz_uids.uid = zzz_connections.uid
            AND zzz_connections.private);

