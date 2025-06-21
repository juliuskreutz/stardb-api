INSERT INTO gacha_banners (
    game_id,
    gacha_type,
    banner_id,
    title,
    internal_name,
    version,
    rate_up_5_stars,
    rate_up_4_stars,
    start_time,
    end_time,
    timezone_dependant,
    disabled
) VALUES (
     $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12
 )
RETURNING id;
