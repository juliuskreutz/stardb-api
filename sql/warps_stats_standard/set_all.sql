INSERT INTO warps_stats_standard (uid, count_rank, luck_4, luck_4_rank, luck_5, luck_5_rank)
SELECT
    *
FROM
    UNNEST($1::integer[], $2::integer[], $3::double precision[], $4::integer[], $5::double precision[], $6::integer[])
ON CONFLICT (uid)
    DO UPDATE SET
        count_rank = EXCLUDED.count_rank,
        luck_4 = EXCLUDED.luck_4,
        luck_4_rank = EXCLUDED.luck_4_rank,
        luck_5 = EXCLUDED.luck_5,
        luck_5_rank = EXCLUDED.luck_5_rank;

