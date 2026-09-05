//! Periodically rebuilds ZZZ population percentiles in database batches.

use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;

use crate::{
    database::{self, zzz::signals_stats_global::DbSignalsStatGlobal},
    gacha::global_stats::{calculate_percentiles, PercentileInput, UPDATE_BATCH_SIZE},
};

/// Minimum history size required before a user appears in global rankings.
const MINIMUM_RANKED_PULLS: i32 = 50;

/// Spawns the hourly ZZZ global-stat updater.
pub async fn spawn(pool: PgPool) {
    super::spawn_periodic(
        "signals_stats",
        Duration::from_secs(3600),
        Duration::from_secs(30),
        move || {
            let pool = pool.clone();
            async move { update(&pool).await }
        },
    );
}

/// Refreshes supported pools sequentially, stopping at the first failure.
/// Previously written batches remain committed for the next idempotent retry.
async fn update(pool: &PgPool) -> Result<()> {
    info!("Starting standard");
    standard(pool).await?;
    info!("Starting special");
    special(pool).await?;
    info!("Starting w_engine");
    w_engine(pool).await?;
    info!("Starting exclusive_rescreening");
    exclusive_rescreening(pool).await?;
    info!("Starting w_engine_reverberation");
    w_engine_reverberation(pool).await?;
    info!("Starting bangboo");
    bangboo(pool).await?;
    Ok(())
}

/// Partitions count-joined rows into calculated and stale global records.
///
/// ZZZ historically removes global rows for histories below 50 pulls or with
/// no S-rank luck value. Returning those UIDs separately preserves that
/// behavior while allowing one bulk delete instead of one delete per UID.
fn calculate_stats(
    stats: Vec<database::zzz::signals_stats::DbSignalsStatCount>,
) -> (Vec<DbSignalsStatGlobal>, Vec<i32>) {
    let mut eligible = Vec::with_capacity(stats.len());
    let mut ineligible = Vec::new();
    for stat in stats {
        let count = stat.signal_count.unwrap_or(0) as i32;
        if count < MINIMUM_RANKED_PULLS || stat.luck_s == 0.0 {
            ineligible.push(stat.uid);
        } else {
            eligible.push(PercentileInput {
                uid: stat.uid,
                count,
                luck_low: stat.luck_a,
                luck_high: stat.luck_s,
            });
        }
    }

    let calculated = calculate_percentiles(eligible)
        .into_iter()
        .map(|stat| DbSignalsStatGlobal {
            uid: stat.uid,
            count_percentile: stat.count,
            luck_a_percentile: stat.luck_low,
            luck_s_percentile: stat.luck_high,
        })
        .collect();
    (calculated, ineligible)
}

/// Refreshes the standard pool through the shared within-game batch path.
async fn standard(pool: &PgPool) -> Result<()> {
    refresh(crate::ZzzGachaType::Standard, pool).await
}

/// Refreshes the special pool through the shared within-game batch path.
async fn special(pool: &PgPool) -> Result<()> {
    refresh(crate::ZzzGachaType::Special, pool).await
}

/// Refreshes the w engine pool through the shared within-game batch path.
async fn w_engine(pool: &PgPool) -> Result<()> {
    refresh(crate::ZzzGachaType::WEngine, pool).await
}

/// Refreshes the exclusive rescreening pool through the shared within-game batch path.
async fn exclusive_rescreening(pool: &PgPool) -> Result<()> {
    refresh(crate::ZzzGachaType::ExclusiveRescreening, pool).await
}

/// Refreshes the w engine reverberation pool through the shared within-game batch path.
async fn w_engine_reverberation(pool: &PgPool) -> Result<()> {
    refresh(crate::ZzzGachaType::WEngineReverberation, pool).await
}

/// Refreshes the bangboo pool through the shared within-game batch path.
async fn bangboo(pool: &PgPool) -> Result<()> {
    refresh(crate::ZzzGachaType::Bangboo, pool).await
}

/// Fetches one pool’s population and writes calculated percentiles in bounded batches.
/// Deletes ineligible UID rows before upserting eligible results; these operations
/// are separately committed, so failures are repaired by a later retry.
async fn refresh(kind: crate::ZzzGachaType, pool: &PgPool) -> Result<()> {
    let (stats, ineligible) =
        calculate_stats(database::zzz::signals_stats::get_all_by_pool(kind, pool).await?);
    database::zzz::signals_stats_global::delete_bulk_by_pool(kind, &ineligible, pool).await?;
    for batch in stats.chunks(UPDATE_BATCH_SIZE) {
        database::zzz::signals_stats_global::set_bulk_by_pool(kind, batch, pool).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[test]
    fn partitions_every_ineligible_reason_before_ranking() {
        let (calculated, ineligible) = calculate_stats(vec![
            database::zzz::signals_stats::DbSignalsStatCount {
                uid: 1,
                luck_a: 8.0,
                luck_s: 70.0,
                signal_count: Some(50),
            },
            database::zzz::signals_stats::DbSignalsStatCount {
                uid: 2,
                luck_a: 6.0,
                luck_s: 80.0,
                signal_count: Some(49),
            },
            database::zzz::signals_stats::DbSignalsStatCount {
                uid: 3,
                luck_a: 7.0,
                luck_s: 0.0,
                signal_count: Some(100),
            },
        ]);

        assert_eq!(ineligible, vec![2, 3]);
        assert_eq!(calculated.len(), 1);
        assert_eq!(calculated[0].uid, 1);
        assert_eq!(calculated[0].count_percentile, 0.0);
    }

    #[actix_web::test]
    async fn aggregate_count_read_and_bulk_write_delete_round_trip() {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        let suffix = (Uuid::new_v4().as_u128() % 10_000_000) as i32;
        let first_uid = 1_500_000_000 + suffix;
        let second_uid = first_uid + 10_000_000;
        let stale_uid = second_uid + 10_000_000;
        let item = 1_700_000_000 + suffix;
        sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1), ($2), ($3)")
            .bind(first_uid)
            .bind(second_uid)
            .bind(stale_uid)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO zzz_characters (id, rarity) VALUES ($1, 4)")
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO zzz_signals_stats_standard (uid, luck_a, luck_s) VALUES ($1, 8, 70), ($2, 6, 80), ($3, 7, 75)",
        )
        .bind(first_uid)
        .bind(second_uid)
        .bind(stale_uid)
        .execute(&pool)
        .await
        .unwrap();

        for (uid, count) in [
            (first_uid, 60_i32),
            (second_uid, 50_i32),
            (stale_uid, 49_i32),
        ] {
            sqlx::query(
                "INSERT INTO zzz_signals_standard (id, uid, character, timestamp, official) SELECT $1::bigint + pull, $2, $3, to_timestamp(1700000000 + pull), true FROM generate_series(1, $4) AS pull",
            )
            .bind(i64::from(uid) * 1_000)
            .bind(uid)
            .bind(item)
            .bind(count)
            .execute(&pool)
            .await
            .unwrap();
        }

        database::zzz::signals_stats_global::standard::set_bulk(
            &[DbSignalsStatGlobal {
                uid: stale_uid,
                count_percentile: 0.25,
                luck_a_percentile: 0.25,
                luck_s_percentile: 0.25,
            }],
            &pool,
        )
        .await
        .unwrap();
        let source = database::zzz::signals_stats::standard::get_all(&pool)
            .await
            .unwrap();
        assert_eq!(
            source
                .iter()
                .find(|stat| stat.uid == stale_uid)
                .unwrap()
                .signal_count,
            Some(49)
        );

        standard(&pool).await.unwrap();
        let first = database::zzz::signals_stats_global::standard::get_by_uid(first_uid, &pool)
            .await
            .unwrap()
            .unwrap();
        let second = database::zzz::signals_stats_global::standard::get_by_uid(second_uid, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (
                first.count_percentile,
                first.luck_a_percentile,
                first.luck_s_percentile,
            ),
            (0.0, 0.5, 0.0)
        );
        assert_eq!(
            (
                second.count_percentile,
                second.luck_a_percentile,
                second.luck_s_percentile,
            ),
            (0.5, 0.0, 0.5)
        );
        assert!(
            database::zzz::signals_stats_global::standard::get_by_uid(stale_uid, &pool)
                .await
                .unwrap()
                .is_none()
        );

        sqlx::query("DELETE FROM zzz_uids WHERE uid = ANY($1::integer[])")
            .bind(vec![first_uid, second_uid, stale_uid])
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM zzz_characters WHERE id = $1")
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
    }
}
