use anyhow::Result;
use sqlx::PgPool;

use crate::database::warps_stats::{DbWarpsStat, DbWarpsStatCount};

pub struct DbWarpsStatStandard {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
}

impl From<DbWarpsStatStandard> for DbWarpsStat {
    fn from(s: DbWarpsStatStandard) -> Self {
        DbWarpsStat {
            uid: s.uid,
            luck_4: s.luck_4,
            luck_5: s.luck_5,
            win_rate: 0.0,
            win_streak: 0,
            loss_streak: 0,
        }
    }
}

pub async fn set(stat: &DbWarpsStat, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/warps_stats/standard/set.sql",
        stat.uid,
        stat.luck_4,
        stat.luck_5,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbWarpsStatCount>> {
    Ok(sqlx::query_file_as!(
        DbWarpsStatCount,
        "sql/warps_stats/standard/get_all_count.sql"
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Option<DbWarpsStat>> {
    let results = sqlx::query_file_as!(
        DbWarpsStatStandard,
        "sql/warps_stats/standard/get_by_uid.sql",
        uid,
    )
    .fetch_optional(pool)
    .await?;

    Ok(results.map(DbWarpsStat::from))
}
