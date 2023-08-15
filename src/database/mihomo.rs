use chrono::NaiveDateTime;
use sqlx::PgPool;

use anyhow::Result;

pub struct DbMihomo {
    pub uid: i64,
    pub region: String,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: NaiveDateTime,
}

pub async fn set_mihomo(mihomo: &DbMihomo, pool: &PgPool) -> Result<DbMihomo> {
    sqlx::query_as!(
        DbMihomo,
        "
        INSERT INTO
            mihomo(uid, region, name, level, signature, avatar_icon, achievement_count, updated_at)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT
            (uid)
        DO UPDATE SET
            name = EXCLUDED.name,
            level = EXCLUDED.level,
            signature = EXCLUDED.signature,
            avatar_icon = EXCLUDED.avatar_icon,
            achievement_count = EXCLUDED.achievement_count,
            updated_at = EXCLUDED.updated_at
        ",
        mihomo.uid,
        mihomo.region,
        mihomo.name,
        mihomo.level,
        mihomo.signature,
        mihomo.avatar_icon,
        mihomo.achievement_count,
        mihomo.updated_at,
    )
    .execute(pool)
    .await?;

    get_mihomo_by_uid(mihomo.uid, pool).await
}

pub async fn get_mihomo_by_uid(uid: i64, pool: &PgPool) -> Result<DbMihomo> {
    Ok(sqlx::query_as!(
        DbMihomo,
        "
        SELECT
            *
        FROM
            mihomo
        WHERE
            uid = $1
        ",
        uid,
    )
    .fetch_one(pool)
    .await?)
}
