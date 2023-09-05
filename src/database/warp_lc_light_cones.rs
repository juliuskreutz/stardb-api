use anyhow::Result;
use chrono::NaiveDateTime;
use sqlx::PgPool;

pub struct DbWarpLcLightCone {
    pub id: i64,
    pub uid: i64,
    pub light_cone: i32,
    pub name: String,
    pub timestamp: NaiveDateTime,
}

pub async fn set_warp_lc_light_cone(
    warp_lc_light_cone: &DbWarpLcLightCone,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            warp_lc_light_cones(id, uid, light_cone, timestamp)
        VALUES
            ($1, $2, $3, $4)
        ON CONFLICT
            (id, uid)
        DO UPDATE SET
            light_cone = EXCLUDED.light_cone,
            timestamp = EXCLUDED.timestamp
        ",
        warp_lc_light_cone.id,
        warp_lc_light_cone.uid,
        warp_lc_light_cone.light_cone,
        warp_lc_light_cone.timestamp,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_warp_lc_light_cones_by_uid(
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<Vec<DbWarpLcLightCone>> {
    Ok(sqlx::query_as!(
        DbWarpLcLightCone,
        "
        SELECT
            warp_lc_light_cones.*,
            light_cones_text.name
        FROM
            warp_lc_light_cones
        INNER JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cone AND light_cones_text.language = $2
        WHERE
            uid = $1
        ORDER BY
            timestamp
        ",
        uid,
        language,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_warp_lc_light_cone_by_id_and_uid(
    id: i64,
    uid: i64,
    language: &str,
    pool: &PgPool,
) -> Result<DbWarpLcLightCone> {
    Ok(sqlx::query_as!(
        DbWarpLcLightCone,
        "
        SELECT
            warp_lc_light_cones.*,
            light_cones_text.name
        FROM
            warp_lc_light_cones
        INNER JOIN
            light_cones_text
        ON
            light_cones_text.id = light_cone AND light_cones_text.language = $3
        WHERE
            warp_lc_light_cones.id = $1
        AND
            uid = $2
        ",
        id,
        uid,
        language,
    )
    .fetch_one(pool)
    .await?)
}
