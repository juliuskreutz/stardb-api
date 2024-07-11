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
            zzz_signals.uid = zzz_uids.uid)
    AND NOT EXISTS (
        SELECT
            *
        FROM
            zzz_connections
        WHERE
            zzz_connections.uid = zzz_uids.uid
            AND private);

