SELECT
    uid
FROM
    zzz_uids
WHERE
    EXISTS (
        SELECT
            *
        FROM (
            SELECT
                uid
            FROM
                zzz_signals_standard
            UNION ALL
            SELECT
                uid
            FROM
                zzz_signals_special
            UNION ALL
            SELECT
                uid
            FROM
                zzz_signals_w_engine
            UNION ALL
            SELECT
                uid
            FROM
                zzz_signals_bangboo) zzz_signals
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

