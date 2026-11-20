//! Running totals behind achievement completion percentages.
//!
//! A percentage is the share of qualifying players who completed an achievement. Instead
//! of re-aggregating every completion each hour, list writes update two totals in their
//! own transaction: how many qualifying players completed each achievement
//! (`*achievements_percent.completions`) and how many achievements each player completed
//! (`*achievement_completion_totals`). Readers divide the two. A full recount runs when a
//! game has never been counted, when its threshold changed, and weekly, correcting drift
//! from writes that bypass the lists (account deletion, catalog removals) or that land
//! while a recount is reading.

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use sqlx::{PgConnection, PgPool};

use super::achievement_lists::Game;

/// How often a full recount corrects drift once a game's totals exist.
const RECOUNT_INTERVAL_DAYS: i32 = 7;

/// Completions a player needs before they count toward a game's percentages.
fn qualifying_minimum(game: Game) -> i32 {
    match game {
        Game::Hsr => 600,
        Game::Gi | Game::Zzz => 1,
    }
}

/// Apply one player's net list change to the running totals.
///
/// `added` and `removed` must be disjoint and describe completions that changed in the
/// caller's transaction, after its rows were written. Counters are updated in id order
/// so concurrent writers cannot deadlock. Does nothing until the game's first recount.
pub(crate) async fn record_change(
    game: Game,
    username: &str,
    added: &[i32],
    removed: &[i32],
    connection: &mut PgConnection,
) -> Result<()> {
    if added.is_empty() && removed.is_empty() {
        return Ok(());
    }
    let counted: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM achievement_completion_state WHERE game = $1)",
    )
    .bind(game.key())
    .fetch_one(&mut *connection)
    .await?;
    if !counted {
        return Ok(());
    }

    let prefix = game.prefix();
    // Only closed game enums supply identifiers; request data remains bound.
    let current: Vec<i32> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT id FROM {prefix}users_achievements_completed WHERE username = $1"
    )))
    .bind(username)
    .fetch_all(&mut *connection)
    .await?;

    let new_total = current.len();
    let old_total = new_total + removed.len() - added.len();
    let minimum = qualifying_minimum(game) as usize;
    let mut deltas: HashMap<i32, i64> = HashMap::new();
    match (old_total >= minimum, new_total >= minimum) {
        (true, true) => {
            for &id in added {
                *deltas.entry(id).or_default() += 1;
            }
            for &id in removed {
                *deltas.entry(id).or_default() -= 1;
            }
        }
        // Crossing the threshold adds or removes the player's whole list.
        (false, true) => {
            for &id in &current {
                *deltas.entry(id).or_default() += 1;
            }
        }
        (true, false) => {
            let added: HashSet<i32> = added.iter().copied().collect();
            for &id in current
                .iter()
                .filter(|id| !added.contains(id))
                .chain(removed)
            {
                *deltas.entry(id).or_default() -= 1;
            }
        }
        (false, false) => {}
    }

    if new_total == 0 {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DELETE FROM {prefix}achievement_completion_totals WHERE username = $1"
        )))
        .bind(username)
        .execute(&mut *connection)
        .await?;
    } else {
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO {prefix}achievement_completion_totals (username, completed) VALUES ($1, $2) \
             ON CONFLICT (username) DO UPDATE SET completed = EXCLUDED.completed"
        )))
        .bind(username)
        .bind(new_total as i32)
        .execute(&mut *connection)
        .await?;
    }

    let mut deltas: Vec<(i32, i64)> = deltas.into_iter().filter(|&(_, d)| d != 0).collect();
    if deltas.is_empty() {
        return Ok(());
    }
    deltas.sort_unstable();
    let (ids, amounts): (Vec<i32>, Vec<i64>) = deltas.into_iter().unzip();
    // GREATEST keeps a drifted counter at zero rather than failing the user's write;
    // the next recount restores the exact value.
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "WITH changes AS (SELECT id, amount FROM UNNEST($1::int[], $2::bigint[]) AS changes(id, amount)) \
         INSERT INTO {prefix}achievements_percent AS counts (id, percent, completions) \
         SELECT id, 0, GREATEST(amount, 0) FROM changes ORDER BY id \
         ON CONFLICT (id) DO UPDATE SET completions = GREATEST( \
             COALESCE(counts.completions, 0) + (SELECT amount FROM changes WHERE changes.id = EXCLUDED.id), 0)"
    )))
    .bind(&ids)
    .bind(&amounts)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

/// Whether the game has no totals yet, was counted with another threshold, or is due.
pub(crate) async fn recount_due(game: Game, pool: &PgPool) -> Result<bool> {
    let fresh: Option<bool> = sqlx::query_scalar(
        "SELECT minimum_completions = $2 AND recomputed_at > now() - make_interval(days => $3) \
         FROM achievement_completion_state WHERE game = $1",
    )
    .bind(game.key())
    .bind(qualifying_minimum(game))
    .bind(RECOUNT_INTERVAL_DAYS)
    .fetch_optional(pool)
    .await?;
    Ok(fresh != Some(true))
}

/// Recount a game's totals from its completion lists and mark it counted.
///
/// The aggregation reads into temporary tables first, so live rows are only locked for
/// the short swap at the end and list writes are never held up by the scan. The caller
/// owns the transaction.
pub(crate) async fn recount(game: Game, connection: &mut PgConnection) -> Result<()> {
    let prefix = game.prefix();
    let minimum = qualifying_minimum(game);
    // Earlier recounts in the same transaction leave their temporary tables until commit.
    sqlx::query("DROP TABLE IF EXISTS pg_temp.recount_totals, pg_temp.recount_counts")
        .execute(&mut *connection)
        .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE TEMPORARY TABLE recount_totals ON COMMIT DROP AS \
         SELECT username, COUNT(*)::integer AS completed \
         FROM {prefix}users_achievements_completed GROUP BY username"
    )))
    .execute(&mut *connection)
    .await?;
    // CREATE TABLE AS cannot take bind parameters; the minimum is a constant from code.
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE TEMPORARY TABLE recount_counts ON COMMIT DROP AS \
         SELECT c.id, COUNT(*) AS completions \
         FROM {prefix}users_achievements_completed c JOIN recount_totals t USING (username) \
         WHERE t.completed >= {minimum} GROUP BY c.id"
    )))
    .execute(&mut *connection)
    .await?;
    let qualifying: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM recount_totals WHERE completed >= $1")
            .bind(minimum)
            .fetch_one(&mut *connection)
            .await?;

    sqlx::query(sqlx::AssertSqlSafe(format!(
        "DELETE FROM {prefix}achievement_completion_totals t \
         WHERE NOT EXISTS (SELECT 1 FROM recount_totals r WHERE r.username = t.username)"
    )))
    .execute(&mut *connection)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "INSERT INTO {prefix}achievement_completion_totals (username, completed) \
         SELECT username, completed FROM recount_totals ORDER BY username \
         ON CONFLICT (username) DO UPDATE SET completed = EXCLUDED.completed \
         WHERE {prefix}achievement_completion_totals.completed IS DISTINCT FROM EXCLUDED.completed"
    )))
    .execute(&mut *connection)
    .await?;
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "INSERT INTO {prefix}achievements_percent (id, percent, completions) \
         SELECT a.id, COALESCE(COALESCE(r.completions, 0)::float8 / NULLIF($1::float8, 0), 0), \
                COALESCE(r.completions, 0) \
         FROM {prefix}achievements a LEFT JOIN recount_counts r USING (id) ORDER BY a.id \
         ON CONFLICT (id) DO UPDATE SET percent = EXCLUDED.percent, completions = EXCLUDED.completions"
    )))
    .bind(qualifying as f64)
    .execute(&mut *connection)
    .await?;
    sqlx::query(
        "INSERT INTO achievement_completion_state (game, minimum_completions, recomputed_at) \
         VALUES ($1, $2, now()) \
         ON CONFLICT (game) DO UPDATE SET minimum_completions = EXCLUDED.minimum_completions, \
             recomputed_at = EXCLUDED.recomputed_at",
    )
    .bind(game.key())
    .bind(minimum)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sqlx::PgPool;

    use super::{recount, Game};
    use crate::database::achievement_lists::{add_all, apply, delete_all, List};

    /// Non-zero completion counters and per-player totals, as stored.
    type Totals = (BTreeMap<i32, i64>, BTreeMap<String, i32>);

    async fn stored(game: Game, pool: &PgPool) -> Totals {
        let prefix = game.prefix();
        // Test-only identifiers from the closed Game enum.
        let counts = sqlx::query_as::<_, (i32, i64)>(sqlx::AssertSqlSafe(format!(
            "SELECT id, completions FROM {prefix}achievements_percent WHERE completions > 0"
        )))
        .fetch_all(pool)
        .await
        .unwrap();
        let totals = sqlx::query_as::<_, (String, i32)>(sqlx::AssertSqlSafe(format!(
            "SELECT username, completed FROM {prefix}achievement_completion_totals"
        )))
        .fetch_all(pool)
        .await
        .unwrap();
        (counts.into_iter().collect(), totals.into_iter().collect())
    }

    async fn recount_now(game: Game, pool: &PgPool) {
        let mut transaction = pool.begin().await.unwrap();
        recount(game, &mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
    }

    /// The running totals must equal what a full recount computes from the lists.
    async fn assert_matches_recount(step: &str, pool: &PgPool) {
        for game in [Game::Hsr, Game::Gi] {
            let running = stored(game, pool).await;
            recount_now(game, pool).await;
            assert_eq!(running, stored(game, pool).await, "{step}: {}", game.key());
        }
    }

    #[sqlx::test]
    async fn running_totals_match_a_full_recount(pool: PgPool) {
        sqlx::query("INSERT INTO achievement_series (id, priority) VALUES (1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO achievements (id, series, jades, hidden, priority, impossible) \
             SELECT i, 1, 5, false, 0, false FROM generate_series(1, 610) i",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO gi_achievement_series (id, priority) VALUES (1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO gi_achievements (id, series, primogems, hidden, priority, impossible) \
             SELECT i, 1, 5, false, 0, false FROM generate_series(1, 5) i",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO users (username, password) VALUES ('alice', ''), ('bob', ''), ('carol', '')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let ids = |range: std::ops::RangeInclusive<i32>| range.collect::<Vec<_>>();

        // Completions written before the first recount are picked up by it.
        add_all(Game::Hsr, List::Completed, "alice", &ids(1..=605), &pool)
            .await
            .unwrap();
        add_all(Game::Gi, List::Completed, "carol", &ids(1..=2), &pool)
            .await
            .unwrap();
        recount_now(Game::Hsr, &pool).await;
        recount_now(Game::Gi, &pool).await;
        assert_eq!(stored(Game::Hsr, &pool).await.0[&1], 1, "alice qualifies");

        add_all(Game::Hsr, List::Completed, "bob", &ids(1..=599), &pool)
            .await
            .unwrap();
        assert_eq!(stored(Game::Hsr, &pool).await.0[&1], 1, "bob is below 600");
        assert_matches_recount("below the threshold", &pool).await;

        add_all(Game::Hsr, List::Completed, "bob", &[600], &pool)
            .await
            .unwrap();
        assert_eq!(stored(Game::Hsr, &pool).await.0[&1], 2, "bob crossed 600");
        assert_matches_recount("crossing the threshold", &pool).await;

        let mut transaction = pool.begin().await.unwrap();
        apply(
            Game::Hsr,
            List::Completed,
            "alice",
            &ids(301..=610),
            true,
            &mut transaction,
        )
        .await
        .unwrap();
        transaction.commit().await.unwrap();
        assert_matches_recount("replacing import below the threshold", &pool).await;

        delete_all(Game::Hsr, List::Completed, "bob", &ids(1..=5), &pool)
            .await
            .unwrap();
        assert_matches_recount("deleting below the threshold", &pool).await;

        add_all(Game::Gi, List::Completed, "bob", &ids(2..=5), &pool)
            .await
            .unwrap();
        delete_all(Game::Gi, List::Completed, "carol", &ids(1..=2), &pool)
            .await
            .unwrap();
        assert_matches_recount("gi adds and a player leaving", &pool).await;

        // Favorites never touch completion totals.
        let before = stored(Game::Hsr, &pool).await;
        add_all(Game::Hsr, List::Favorites, "carol", &ids(1..=10), &pool)
            .await
            .unwrap();
        assert_eq!(before, stored(Game::Hsr, &pool).await);
    }
}
