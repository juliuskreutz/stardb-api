//! Backfill for the per-pool `pull_count` stored with each player's stats.
//!
//! Recalculation writes the count from the rows it already reads, so the hourly
//! updaters never scan pull tables. Rows written before the column existed, and rows
//! whose recalculation failed after their pulls changed ([`invalidate`]), hold NULL;
//! this fills them from each player's indexed pull rows, a chunk at a time.

use anyhow::Result;
use sqlx::PgPool;

use crate::gacha::imports::PullGame;

/// Players counted per statement, keeping each backfill statement short.
const CHUNK: i64 = 1_000;

/// Fills NULL counts in `stats_table` from `pull_table` and returns how many rows were set.
///
/// Only NULL counts are written, so a concurrent recalculation that already stored a
/// fresh count is never overwritten. Players without pulls get 0. Both table names
/// come from the closed stats registries, never from request data. NULL rows are found
/// through each table's `pull_count IS NULL` partial index, so every chunk and the
/// steady-state check read only the rows still missing.
pub(crate) async fn backfill(stats_table: &str, pull_table: &str, pool: &PgPool) -> Result<u64> {
    let mut filled = 0;
    loop {
        let rows = sqlx::query(sqlx::AssertSqlSafe(format!(
            "WITH missing AS (SELECT uid FROM {stats_table} WHERE pull_count IS NULL LIMIT {CHUNK}) \
             UPDATE {stats_table} AS stats \
             SET pull_count = (SELECT COUNT(*) FROM {pull_table} AS pulls WHERE pulls.uid = stats.uid) \
             FROM missing WHERE stats.uid = missing.uid AND stats.pull_count IS NULL"
        )))
        .execute(pool)
        .await?
        .rows_affected();
        if rows == 0 {
            return Ok(filled);
        }
        filled += rows;
    }
}

/// Clears one UID's stored pull counts for a game so the next backfill recounts them.
///
/// For when pulls may have changed but recalculation failed: the stale counts would
/// otherwise keep ranking the player on pulls they no longer have.
pub(crate) async fn invalidate(game: PullGame, uid: i32, pool: &PgPool) -> Result<()> {
    let tables: &[&str] = match game {
        PullGame::Hsr => &[
            "warps_stats_standard",
            "warps_stats_special",
            "warps_stats_lc",
            "warps_stats_collab",
            "warps_stats_collab_lc",
        ],
        PullGame::Gi => &[
            "gi_wishes_stats_standard",
            "gi_wishes_stats_character",
            "gi_wishes_stats_weapon",
            "gi_wishes_stats_chronicled",
        ],
        PullGame::Zzz => &[
            "zzz_signals_stats_standard",
            "zzz_signals_stats_special",
            "zzz_signals_stats_w_engine",
            "zzz_signals_stats_bangboo",
            "zzz_signals_stats_exclusive_rescreening",
            "zzz_signals_stats_w_engine_reverberation",
        ],
    };
    for table in tables {
        // Fixed table names from the list above; the uid stays bound.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE {table} SET pull_count = NULL WHERE uid = $1"
        )))
        .bind(uid)
        .execute(pool)
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    async fn pool() -> sqlx::PgPool {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    /// Inserts `count` standard-pool signals for `uid` using the given character.
    async fn add_signals(uid: i32, first_id: i64, count: i32, item: i32, pool: &sqlx::PgPool) {
        sqlx::query(
            "INSERT INTO zzz_signals_standard (id, uid, character, timestamp, official) \
             SELECT $1::bigint + pull, $2, $3, to_timestamp(1700000000 + pull), true \
             FROM generate_series(1, $4) AS pull",
        )
        .bind(first_id)
        .bind(uid)
        .bind(item)
        .bind(count)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn stored_count(table: &str, uid: i32, pool: &sqlx::PgPool) -> Option<i32> {
        // Test-only table names from the fixed list below, never request data.
        sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT pull_count FROM {table} WHERE uid = $1"
        )))
        .bind(uid)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[actix_web::test]
    async fn backfill_counts_missing_rows_and_keeps_stored_counts() {
        let pool = pool().await;
        let suffix = (Uuid::new_v4().as_u128() % 10_000_000) as i32;
        let (with_pulls, without_pulls, already_stored) = (
            1_100_000_000 + suffix,
            1_110_000_000 + suffix,
            1_120_000_000 + suffix,
        );
        let item = 1_900_000_000 + suffix;
        sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1), ($2), ($3)")
            .bind(with_pulls)
            .bind(without_pulls)
            .bind(already_stored)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO zzz_characters (id, rarity) VALUES ($1, 3)")
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO zzz_signals_stats_standard (uid, luck_a, luck_s, pull_count) \
             VALUES ($1, 0, 0, NULL), ($2, 0, 0, NULL), ($3, 0, 0, 5)",
        )
        .bind(with_pulls)
        .bind(without_pulls)
        .bind(already_stored)
        .execute(&pool)
        .await
        .unwrap();
        add_signals(with_pulls, i64::from(with_pulls) * 1_000, 7, item, &pool).await;
        add_signals(
            already_stored,
            i64::from(already_stored) * 1_000,
            3,
            item,
            &pool,
        )
        .await;

        super::backfill("zzz_signals_stats_standard", "zzz_signals_standard", &pool)
            .await
            .unwrap();

        let table = "zzz_signals_stats_standard";
        assert_eq!(stored_count(table, with_pulls, &pool).await, Some(7));
        assert_eq!(stored_count(table, without_pulls, &pool).await, Some(0));
        // A count written by recalculation is never replaced by the backfill.
        assert_eq!(stored_count(table, already_stored, &pool).await, Some(5));
    }

    #[actix_web::test]
    async fn recalculation_stores_the_pull_count_of_every_pool() {
        let pool = pool().await;
        let suffix = (Uuid::new_v4().as_u128() % 10_000_000) as i32;
        let uid = 1_130_000_000 + suffix;
        let item = 1_910_000_000 + suffix;
        sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1)")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO zzz_characters (id, rarity) VALUES ($1, 3)")
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        add_signals(uid, i64::from(uid) * 1_000, 12, item, &pool).await;

        let recalculate = || {
            crate::gacha::imports::recalculate_uid_in_transaction(
                crate::gacha::imports::PullGame::Zzz,
                uid,
                &pool,
            )
        };
        recalculate().await.unwrap();
        assert_eq!(
            stored_count("zzz_signals_stats_standard", uid, &pool).await,
            Some(12)
        );
        // Pools without pulls still store an explicit zero rather than NULL.
        assert_eq!(
            stored_count("zzz_signals_stats_special", uid, &pool).await,
            Some(0)
        );

        add_signals(uid, i64::from(uid) * 1_000 + 500, 4, item, &pool).await;
        recalculate().await.unwrap();
        assert_eq!(
            stored_count("zzz_signals_stats_standard", uid, &pool).await,
            Some(16)
        );
    }
}
