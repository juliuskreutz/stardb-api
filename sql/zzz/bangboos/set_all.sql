INSERT INTO zzz_bangboos (id, rarity)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::integer[])
ON CONFLICT (id)
    DO UPDATE SET
        rarity = EXCLUDED.rarity;

