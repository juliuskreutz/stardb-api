use anyhow::Result;
use sqlx::PgPool;

pub async fn update_warps_avg(pool: &PgPool) -> Result<()> {
    let mut transaction = pool.begin().await?;

    sqlx::query!("TRUNCATE warps_stats")
        .execute(&mut *transaction)
        .await?;

    sqlx::query!("TRUNCATE warps_stats_4")
        .execute(&mut *transaction)
        .await?;

    sqlx::query!("TRUNCATE warps_stats_5")
        .execute(&mut *transaction)
        .await?;

    sqlx::query!(
        "
        INSERT INTO
            warps_stats (uid, gacha_type, COUNT, RANK)
        WITH
            warps_stats AS (
                SELECT
                    uid,
                    gacha_type,
                    count(*)
                FROM
                    warps
                GROUP BY
                    uid,
                    gacha_type
            )
        SELECT
            uid,
            gacha_type,
            COUNT,
            rank() OVER (
                PARTITION BY
                    gacha_type
                ORDER BY
                    COUNT DESC
            )
        FROM
            warps_stats;
        "
    )
    .execute(&mut *transaction)
    .await?;

    sqlx::query!(
        "
        WITH
            warps_rare AS (
                SELECT
                    warps.*,
                    (
                        coalesce(characters.rarity, light_cones.rarity) = 4
                    )::int AS rare4,
                    (
                        coalesce(characters.rarity, light_cones.rarity) = 5
                    )::int AS rare5
                FROM
                    warps
                    LEFT JOIN characters ON characters.id = character
                    LEFT JOIN light_cones ON light_cones.id = light_cone
                ORDER BY
                    id
            ),
            warps_sum AS (
                SELECT
                    id,
                    uid,
                    gacha_type,
                    sum(rare4) OVER w sum4,
                    sum(rare5) OVER w sum5,
                    lead(rare4) OVER w lead4,
                    lead(rare5) OVER w lead5
                FROM
                    warps_rare
                WINDOW
                    w AS (
                        PARTITION BY
                            uid,
                            gacha_type
                        ORDER BY
                            id
                    )
            ),
            warps_count_4 AS (
                SELECT
                    uid,
                    gacha_type,
                    count(id)
                FROM
                    warps_sum
                GROUP BY
                    uid,
                    gacha_type,
                    sum4
                HAVING
                    max(lead4) = 1
            ),
            warps_count_5 AS (
                SELECT
                    uid,
                    gacha_type,
                    count(id)
                FROM
                    warps_sum
                GROUP BY
                    uid,
                    gacha_type,
                    sum5
                HAVING
                    max(lead5) = 1
            ),
            _ AS (
                INSERT INTO
                    warps_stats_4 (uid, gacha_type, COUNT, AVG, median, rank_count, rank_avg, rank_median)
                SELECT
                    uid,
                    gacha_type,
                    count(*),
                    avg(COUNT),
                    PERCENTILE_CONT(0.5) WITHIN GROUP (
                        ORDER BY
                            COUNT
                    ) median,
                    RANK() OVER (
                        PARTITION BY
                            gacha_type
                        ORDER BY
                            count(*) DESC
                    ) rank_count,
                    RANK() OVER (
                        PARTITION BY
                            gacha_type
                        ORDER BY
                            avg(COUNT)
                    ) rank_avg,
                    RANK() OVER (
                        PARTITION BY
                            gacha_type
                        ORDER BY
                            PERCENTILE_CONT(0.5) WITHIN GROUP (
                                ORDER BY
                                    COUNT
                            )
                    ) rank_median
                FROM
                    warps_count_4
                GROUP BY
                    uid,
                    gacha_type
            )
        INSERT INTO
            warps_stats_5 (uid, gacha_type, COUNT, AVG, median, rank_count, rank_avg, rank_median)
        SELECT
            uid,
            gacha_type,
            count(*),
            avg(COUNT),
            PERCENTILE_CONT(0.5) WITHIN GROUP (
                ORDER BY
                    COUNT
            ) median,
            RANK() OVER (
                PARTITION BY
                    gacha_type
                ORDER BY
                    count(*) DESC
            ) rank_count,
            RANK() OVER (
                PARTITION BY
                    gacha_type
                ORDER BY
                    avg(COUNT)
            ) rank_avg,
            RANK() OVER (
                PARTITION BY
                    gacha_type
                ORDER BY
                    PERCENTILE_CONT(0.5) WITHIN GROUP (
                        ORDER BY
                            COUNT
                    )
            ) rank_median
        FROM
            warps_count_5
        GROUP BY
            uid,
            gacha_type
        HAVING
            CASE 
                WHEN gacha_type = 'departure' THEN TRUE
                WHEN gacha_type = 'standard' OR gacha_type = 'lc' THEN count(*) >= 5
                ELSE count(*) >= 10
            END
        "
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

pub struct DbWarpsStatsUidGachaType {
    pub uid: i32,
    pub gacha_type: String,
    pub count: i64,
    pub avg: f64,
    pub median: i64,
    pub sum: i64,
    pub rank_count: i64,
    pub rank_avg: i64,
    pub rank_median: i64,
    pub rank_sum: i64,
}

pub async fn get_warps_stats_4_by_uid(
    uid: i32,
    pool: &PgPool,
) -> Result<Vec<DbWarpsStatsUidGachaType>> {
    let warps_stats = sqlx::query_as!(
        DbWarpsStatsUidGachaType,
        "
        SELECT
            warps_stats_4.*,
            warps_stats.count sum,
            warps_stats.rank rank_sum
        FROM
            warps_stats
        LEFT JOIN
            warps_stats_4 
        ON 
            warps_stats_4.uid = warps_stats.uid
        AND
            warps_stats_4.gacha_type = warps_stats.gacha_type
        WHERE
            warps_stats_4.uid = $1
        ",
        uid,
    )
    .fetch_all(pool)
    .await?;

    Ok(warps_stats)
}

pub async fn get_warps_stats_5_by_uid(
    uid: i32,
    pool: &PgPool,
) -> Result<Vec<DbWarpsStatsUidGachaType>> {
    let warps_stats = sqlx::query_as!(
        DbWarpsStatsUidGachaType,
        "
        SELECT
            warps_stats_5.*,
            warps_stats.count sum,
            warps_stats.rank rank_sum
        FROM
            warps_stats
        LEFT JOIN
            warps_stats_5 
        ON 
            warps_stats_5.uid = warps_stats.uid
        AND
            warps_stats_5.gacha_type = warps_stats.gacha_type
        WHERE
            warps_stats_5.uid = $1
        ",
        uid,
    )
    .fetch_all(pool)
    .await?;

    Ok(warps_stats)
}

pub struct DbWarpsStatsUid {
    pub uid: i32,
    pub sum: Option<i64>,
    pub rank: Option<i64>,
}

pub async fn get_warps_stats_by_uid(uid: i32, pool: &PgPool) -> Result<DbWarpsStatsUid> {
    let warps_stats = sqlx::query_as!(
        DbWarpsStatsUid,
        "
        SELECT
            *
        FROM (
            SELECT
                uid,
                sum(COUNT),
                rank() OVER (
                    ORDER BY
                        sum(COUNT) DESC
                )
            FROM
                warps_stats
            GROUP BY
                uid
        ) x
        WHERE
            uid = $1
        ",
        uid,
    )
    .fetch_one(pool)
    .await?;

    Ok(warps_stats)
}

pub struct DbWarpsStatsGachaType {
    pub gacha_type: String,
    pub total: Option<i64>,
}

pub async fn get_warps_stats_gacha_type(pool: &PgPool) -> Result<Vec<DbWarpsStatsGachaType>> {
    let warps_stats = sqlx::query_as!(
        DbWarpsStatsGachaType,
        "
        SELECT
            gacha_type,
            count(distinct uid) total
        FROM 
            warps_stats
        GROUP BY
            gacha_type
        "
    )
    .fetch_all(pool)
    .await?;

    Ok(warps_stats)
}

pub struct DbWarpsStats {
    pub total: Option<i64>,
}

pub async fn get_warps_stats(pool: &PgPool) -> Result<DbWarpsStats> {
    let warps_stats = sqlx::query_as!(
        DbWarpsStats,
        "
        SELECT
            count(distinct uid) total
        FROM 
            warps_stats
        "
    )
    .fetch_one(pool)
    .await?;

    Ok(warps_stats)
}
