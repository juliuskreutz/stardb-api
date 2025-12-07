use anyhow::Result;
use sqlx::PgPool;

use crate::database::warps_stats::{DbWarpsStat, DbWarpsStatCount};

pub async fn set(stat: &DbWarpsStat, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats/collab_lc/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
        stat.win_rate,
        stat.win_streak,
        stat.loss_streak,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatCount>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatCount,
        "sql/warps_stats/collab_lc/get_all_count.sql"
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStat>> {
    Ok(
        sqlx::query_file_as!(DbWarpsStat, "sql/warps_stats/collab_lc/get_by_uid.sql", uid)
            .fetch_optional(pool)
            .await?,
    )
}
