SELECT
    count(*)
FROM
    mihomo
WHERE
    EXISTS (
        SELECT
            *
        FROM (
            SELECT
                uid
            FROM
                warps_departure
            UNION ALL
            SELECT
                uid
            FROM
                warps_standard
            UNION ALL
            SELECT
                uid
            FROM
                warps_special
            UNION ALL
            SELECT
                uid
            FROM
                warps_lc) warps
        WHERE
            mihomo.uid = warps.uid);

