SELECT
    count(*)
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
            zzz_uids.uid = zzz_signals.uid);

