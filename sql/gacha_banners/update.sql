UPDATE gacha_banners
SET
    game_id = $1,
    gacha_type = $2,
    banner_id = $3,
    title = $4,
    internal_name = $5,
    version = $6,
    rate_up_5_stars = $7,
    rate_up_4_stars = $8,
    start_time = $9,
    end_time = $10,
    timezone_dependant = $11,
    disabled = $12
WHERE id = $13;
