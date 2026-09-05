//! HSR population-percentile persistence and pool-specific routing.

pub mod collab;
pub mod collab_lc;
pub mod lc;
pub mod special;
pub mod standard;

pub struct DbWarpsStatGlobal {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_4_percentile: f64,
    pub luck_5_percentile: f64,
}

macro_rules! global_registry { ($( $variant:ident => $module:ident ),* $(,)?) => {
/// Upserts the supplied percentile rows in one pool; an empty slice is a no-op.
/// Only listed UIDs are changed, and database failures propagate. Unsupported stat pools are errors.
pub(crate) async fn set_bulk_by_pool(kind:crate::GachaType,rows:&[DbWarpsStatGlobal],pool:&sqlx::PgPool)->anyhow::Result<()> { match kind {$(crate::GachaType::$variant => $module::set_bulk(rows,pool).await,)*_ => anyhow::bail!("pool has no calculated stats"),}}
};}
global_registry! {Standard => standard,Special => special,Lc => lc,Collab => collab,CollabLc => collab_lc}

#[cfg(test)]
mod bulk_write_tests {
    use sqlx::PgPool;

    #[sqlx::test]
    async fn sql_performance_all_percentile_pools_skip_unchanged_rows(pool: PgPool) {
        sqlx::query("INSERT INTO mihomo(uid, region, name, level, signature, avatar_icon, achievement_count)
            SELECT i, 'na', 'seed', 1, '', '', 0 FROM generate_series(1, 1000) i")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO gi_profiles(uid, name) SELECT uid, name FROM mihomo")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO zzz_uids(uid) SELECT uid FROM mihomo")
            .execute(&pool)
            .await
            .unwrap();
        // SQLx must preserve required stats fields and optional counts even when
        // PostgreSQL chooses a different join direction on populated tables.
        use crate::ZzzGachaType::*;
        for (kind, table) in [
            (Standard, "zzz_signals_stats_standard"),
            (Special, "zzz_signals_stats_special"),
            (WEngine, "zzz_signals_stats_w_engine"),
            (Bangboo, "zzz_signals_stats_bangboo"),
            (
                ExclusiveRescreening,
                "zzz_signals_stats_exclusive_rescreening",
            ),
            (
                WEngineReverberation,
                "zzz_signals_stats_w_engine_reverberation",
            ),
        ] {
            let (extra_columns, extra_values) = if matches!(kind, Standard | Bangboo) {
                ("", "")
            } else {
                (", win_rate, win_streak, loss_streak", ", 0.5, 1, 0")
            };
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "INSERT INTO {table}(uid, luck_a, luck_s{extra_columns}) VALUES (1, 8, 70{extra_values})"
            )))
            .execute(&pool)
            .await
            .unwrap();
            let rows = crate::database::zzz::signals_stats::get_all_by_pool(kind, &pool)
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(
                (
                    rows[0].uid,
                    rows[0].luck_a,
                    rows[0].luck_s,
                    rows[0].signal_count
                ),
                (1, 8.0, 70.0, None)
            );
        }
        // Run every production SQL file so uncommon/collab pools cannot miss the guard.
        let queries = [
            include_str!("../../../sql/gi/wishes_stats_global/character/set_bulk.sql"),
            include_str!("../../../sql/gi/wishes_stats_global/chronicled/set_bulk.sql"),
            include_str!("../../../sql/gi/wishes_stats_global/standard/set_bulk.sql"),
            include_str!("../../../sql/gi/wishes_stats_global/weapon/set_bulk.sql"),
            include_str!("../../../sql/warps_stats_global/collab/set_bulk.sql"),
            include_str!("../../../sql/warps_stats_global/collab_lc/set_bulk.sql"),
            include_str!("../../../sql/warps_stats_global/lc/set_bulk.sql"),
            include_str!("../../../sql/warps_stats_global/special/set_bulk.sql"),
            include_str!("../../../sql/warps_stats_global/standard/set_bulk.sql"),
            include_str!("../../../sql/zzz/signals_stats_global/bangboo/set_bulk.sql"),
            include_str!(
                "../../../sql/zzz/signals_stats_global/exclusive_rescreening/set_bulk.sql"
            ),
            include_str!("../../../sql/zzz/signals_stats_global/special/set_bulk.sql"),
            include_str!("../../../sql/zzz/signals_stats_global/standard/set_bulk.sql"),
            include_str!("../../../sql/zzz/signals_stats_global/w_engine/set_bulk.sql"),
            include_str!(
                "../../../sql/zzz/signals_stats_global/w_engine_reverberation/set_bulk.sql"
            ),
        ];
        let uids: Vec<i32> = (1..=1000).collect();
        for query in queries {
            let mut counts = vec![0.5_f64; 1000];
            let mut luck_low = vec![0.25_f64; 1000];
            let mut luck_high = vec![0.75_f64; 1000];
            for (pass, expected) in [(0, 1000), (1, 0), (2, 1), (3, 1), (4, 1), (5, 0)] {
                match pass {
                    2 => counts[0] = 0.6,
                    3 => luck_low[0] = 0.3,
                    4 => luck_high[0] = 0.8,
                    _ => {}
                }
                let changed = sqlx::query(query)
                    .bind(&uids)
                    .bind(&counts)
                    .bind(&luck_low)
                    .bind(&luck_high)
                    .execute(&pool)
                    .await
                    .unwrap()
                    .rows_affected();
                assert_eq!(changed, expected, "pass {pass}: {query}");
            }
            // Verify actual persisted values, not only the reported row count.
            let table = query.split_whitespace().nth(2).unwrap();
            let columns: Vec<_> = query
                .split('(')
                .nth(1)
                .unwrap()
                .split(')')
                .next()
                .unwrap()
                .split(", ")
                .skip(1)
                .collect();
            let stored: (f64, f64, f64) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
                "SELECT {} FROM {table} WHERE uid = 1",
                columns.join(", ")
            )))
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(stored, (0.6, 0.3, 0.8));
            let empty: Vec<i32> = vec![];
            let values: Vec<f64> = vec![];
            assert_eq!(
                sqlx::query(query)
                    .bind(&empty)
                    .bind(&values)
                    .bind(&values)
                    .bind(&values)
                    .execute(&pool)
                    .await
                    .unwrap()
                    .rows_affected(),
                0
            );
        }
    }
}
