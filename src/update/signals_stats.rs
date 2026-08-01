//! Periodically ranks eligible ZZZ signal histories against one another.
//!
//! Per-user luck values are calculated during imports or queued recalculation.
//! This module performs the slower population-wide pass that turns those values
//! into percentiles for tracker responses.

use std::{
    cmp::Reverse,
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::database;

/// Minimum history size required before a user appears in global rankings.
const MINIMUM_RANKED_PULLS: i32 = 50;

#[derive(Clone, Copy)]
struct PercentileInput {
    uid: i32,
    count: i32,
    luck_a: f64,
    luck_s: f64,
}

struct PercentileResult {
    uid: i32,
    count: f64,
    luck_a: f64,
    luck_s: f64,
}

/// Spawns the hourly global-percentile updater.
///
/// A failed pass retries immediately. Successful passes wait for the next
/// interval tick, preserving the updater's existing recovery behavior.
pub async fn spawn(pool: PgPool) {
    actix::Arbiter::new().spawn(async move {
        let mut success = true;
        let mut interval = rt::time::interval(Duration::from_secs(60 * 60));

        loop {
            if success {
                interval.tick().await;
            }

            let start = Instant::now();
            if let Err(error) = update(&pool).await {
                error!(
                    "Signals stats update failed with {error} in {}s",
                    start.elapsed().as_secs_f64()
                );
                success = false;
            } else {
                info!(
                    "Signals stats update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
                success = true;
            }
        }
    });
}

/// Rebuilds global percentiles for every ZZZ pool in a predictable order.
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

/// Converts eligible per-user values into the repository's zero-based
/// percentile representation.
///
/// Pull count sorts descending because a larger history ranks first. Luck sorts
/// ascending because a lower average pity count is better. The calculation
/// intentionally preserves the existing `rank / population_size` convention.
fn calculate_percentiles(inputs: Vec<PercentileInput>) -> Vec<PercentileResult> {
    let mut by_count = inputs.clone();
    by_count.sort_unstable_by_key(|value| Reverse(value.count));

    let mut by_luck_a = inputs.clone();
    by_luck_a.sort_unstable_by(|left, right| left.luck_a.total_cmp(&right.luck_a));

    let mut by_luck_s = inputs;
    by_luck_s.sort_unstable_by(|left, right| left.luck_s.total_cmp(&right.luck_s));

    let count_ranks: HashMap<_, _> = by_count
        .iter()
        .enumerate()
        .map(|(rank, value)| (value.uid, rank))
        .collect();
    let luck_a_ranks: HashMap<_, _> = by_luck_a
        .iter()
        .enumerate()
        .map(|(rank, value)| (value.uid, rank))
        .collect();
    let luck_s_ranks: HashMap<_, _> = by_luck_s
        .iter()
        .enumerate()
        .map(|(rank, value)| (value.uid, rank))
        .collect();

    let population = by_count.len() as f64;
    by_count
        .into_iter()
        .map(|value| PercentileResult {
            uid: value.uid,
            count: count_ranks[&value.uid] as f64 / population,
            luck_a: luck_a_ranks[&value.uid] as f64 / population,
            luck_s: luck_s_ranks[&value.uid] as f64 / population,
        })
        .collect()
}

/// Returns whether a per-user stat has enough history to rank meaningfully.
fn is_eligible(count: i32, luck_s: f64) -> bool {
    count >= MINIMUM_RANKED_PULLS && luck_s != 0.0
}

/// Refreshes Stable Channel percentiles and removes ineligible rows.
async fn standard(pool: &PgPool) -> Result<()> {
    let mut inputs = Vec::new();
    for stat in database::zzz::signals_stats::standard::get_all(pool).await? {
        let count =
            database::zzz::signals::standard::get_count_by_uid(stat.uid, pool).await? as i32;
        if !is_eligible(count, stat.luck_s) {
            database::zzz::signals_stats_global::standard::delete_by_uid(stat.uid, pool).await?;
            continue;
        }
        inputs.push(PercentileInput {
            uid: stat.uid,
            count,
            luck_a: stat.luck_a,
            luck_s: stat.luck_s,
        });
    }

    for value in calculate_percentiles(inputs) {
        database::zzz::signals_stats_global::standard::set(
            &database::zzz::signals_stats_global::standard::DbSignalsStatGlobalStandard {
                uid: value.uid,
                count_percentile: value.count,
                luck_a_percentile: value.luck_a,
                luck_s_percentile: value.luck_s,
            },
            pool,
        )
        .await?;
    }
    Ok(())
}

/// Refreshes Exclusive Channel percentiles and removes ineligible rows.
async fn special(pool: &PgPool) -> Result<()> {
    let mut inputs = Vec::new();
    for stat in database::zzz::signals_stats::special::get_all(pool).await? {
        let count = database::zzz::signals::special::get_count_by_uid(stat.uid, pool).await? as i32;
        if !is_eligible(count, stat.luck_s) {
            database::zzz::signals_stats_global::special::delete_by_uid(stat.uid, pool).await?;
            continue;
        }
        inputs.push(PercentileInput {
            uid: stat.uid,
            count,
            luck_a: stat.luck_a,
            luck_s: stat.luck_s,
        });
    }

    for value in calculate_percentiles(inputs) {
        database::zzz::signals_stats_global::special::set(
            &database::zzz::signals_stats_global::special::DbSignalsStatGlobalSpecial {
                uid: value.uid,
                count_percentile: value.count,
                luck_a_percentile: value.luck_a,
                luck_s_percentile: value.luck_s,
            },
            pool,
        )
        .await?;
    }
    Ok(())
}

/// Refreshes W-Engine Channel percentiles and removes ineligible rows.
async fn w_engine(pool: &PgPool) -> Result<()> {
    let mut inputs = Vec::new();
    for stat in database::zzz::signals_stats::w_engine::get_all(pool).await? {
        let count =
            database::zzz::signals::w_engine::get_count_by_uid(stat.uid, pool).await? as i32;
        if !is_eligible(count, stat.luck_s) {
            database::zzz::signals_stats_global::w_engine::delete_by_uid(stat.uid, pool).await?;
            continue;
        }
        inputs.push(PercentileInput {
            uid: stat.uid,
            count,
            luck_a: stat.luck_a,
            luck_s: stat.luck_s,
        });
    }

    for value in calculate_percentiles(inputs) {
        database::zzz::signals_stats_global::w_engine::set(
            &database::zzz::signals_stats_global::w_engine::DbSignalsStatGlobalWEngine {
                uid: value.uid,
                count_percentile: value.count,
                luck_a_percentile: value.luck_a,
                luck_s_percentile: value.luck_s,
            },
            pool,
        )
        .await?;
    }
    Ok(())
}

/// Refreshes Exclusive Rescreening percentiles and removes ineligible rows.
async fn exclusive_rescreening(pool: &PgPool) -> Result<()> {
    let mut inputs = Vec::new();
    for stat in database::zzz::signals_stats::exclusive_rescreening::get_all(pool).await? {
        let count = database::zzz::signals::exclusive_rescreening::get_count_by_uid(stat.uid, pool)
            .await? as i32;
        if !is_eligible(count, stat.luck_s) {
            database::zzz::signals_stats_global::exclusive_rescreening::delete_by_uid(
                stat.uid, pool,
            )
            .await?;
            continue;
        }
        inputs.push(PercentileInput {
            uid: stat.uid,
            count,
            luck_a: stat.luck_a,
            luck_s: stat.luck_s,
        });
    }

    for value in calculate_percentiles(inputs) {
        database::zzz::signals_stats_global::exclusive_rescreening::set(
            &database::zzz::signals_stats_global::exclusive_rescreening::DbSignalsStatGlobalExclusiveRescreening {
                uid: value.uid,
                count_percentile: value.count,
                luck_a_percentile: value.luck_a,
                luck_s_percentile: value.luck_s,
            },
            pool,
        ).await?;
    }
    Ok(())
}

/// Refreshes W-Engine Reverberation percentiles and removes ineligible rows.
async fn w_engine_reverberation(pool: &PgPool) -> Result<()> {
    let mut inputs = Vec::new();
    for stat in database::zzz::signals_stats::w_engine_reverberation::get_all(pool).await? {
        let count = database::zzz::signals::w_engine_reverberation::get_count_by_uid(stat.uid, pool)
            .await? as i32;
        if !is_eligible(count, stat.luck_s) {
            database::zzz::signals_stats_global::w_engine_reverberation::delete_by_uid(
                stat.uid, pool,
            )
            .await?;
            continue;
        }
        inputs.push(PercentileInput {
            uid: stat.uid,
            count,
            luck_a: stat.luck_a,
            luck_s: stat.luck_s,
        });
    }

    for value in calculate_percentiles(inputs) {
        database::zzz::signals_stats_global::w_engine_reverberation::set(
            &database::zzz::signals_stats_global::w_engine_reverberation::DbSignalsStatGlobalWEngineReverberation {
                uid: value.uid,
                count_percentile: value.count,
                luck_a_percentile: value.luck_a,
                luck_s_percentile: value.luck_s,
            },
            pool,
        ).await?;
    }
    Ok(())
}

/// Refreshes Bangboo Channel percentiles and removes ineligible rows.
async fn bangboo(pool: &PgPool) -> Result<()> {
    let mut inputs = Vec::new();
    for stat in database::zzz::signals_stats::bangboo::get_all(pool).await? {
        let count = database::zzz::signals::bangboo::get_count_by_uid(stat.uid, pool).await? as i32;
        if !is_eligible(count, stat.luck_s) {
            database::zzz::signals_stats_global::bangboo::delete_by_uid(stat.uid, pool).await?;
            continue;
        }
        inputs.push(PercentileInput {
            uid: stat.uid,
            count,
            luck_a: stat.luck_a,
            luck_s: stat.luck_s,
        });
    }

    for value in calculate_percentiles(inputs) {
        database::zzz::signals_stats_global::bangboo::set(
            &database::zzz::signals_stats_global::bangboo::DbSignalsStatGlobalBangboo {
                uid: value.uid,
                count_percentile: value.count,
                luck_a_percentile: value.luck_a,
                luck_s_percentile: value.luck_s,
            },
            pool,
        )
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_preserve_count_and_luck_sort_directions() {
        let values = calculate_percentiles(vec![
            PercentileInput {
                uid: 1,
                count: 100,
                luck_a: 8.0,
                luck_s: 70.0,
            },
            PercentileInput {
                uid: 2,
                count: 50,
                luck_a: 6.0,
                luck_s: 80.0,
            },
        ]);
        let by_uid: HashMap<_, _> = values.into_iter().map(|value| (value.uid, value)).collect();

        assert_eq!(by_uid[&1].count, 0.0);
        assert_eq!(by_uid[&2].count, 0.5);
        assert_eq!(by_uid[&2].luck_a, 0.0);
        assert_eq!(by_uid[&1].luck_s, 0.0);
    }
}
