//! Periodically rebuilds Genshin population percentiles in database batches.

use std::time::{Duration, Instant};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::{
    database::{self, gi::wishes_stats_global::DbWishesStatGlobal},
    gacha::global_stats::{calculate_percentiles, PercentileInput, UPDATE_BATCH_SIZE},
};

/// Spawns the hourly Genshin global-stat updater.
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
                    "Wishes stats update failed with {error} in {}s",
                    start.elapsed().as_secs_f64()
                );
                success = false;
            } else {
                info!(
                    "Wishes stats update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
                success = true;
            }
        }
    });
}

/// Refreshes every Genshin pool in a predictable order.
async fn update(pool: &PgPool) -> Result<()> {
    info!("Starting standard");
    standard(pool).await?;
    info!("Starting character");
    character(pool).await?;
    info!("Starting weapon");
    weapon(pool).await?;
    info!("Starting chronicled");
    chronicled(pool).await?;
    Ok(())
}

/// Converts count-joined repository rows into the shared percentile shape.
fn calculate_stats(
    stats: Vec<database::gi::wishes_stats::DbWishesStatCount>,
) -> Vec<DbWishesStatGlobal> {
    calculate_percentiles(
        stats
            .into_iter()
            .map(|stat| PercentileInput {
                uid: stat.uid,
                count: stat.wish_count.unwrap_or(0) as i32,
                luck_low: stat.luck_4,
                luck_high: stat.luck_5,
            })
            .collect(),
    )
    .into_iter()
    .map(|stat| DbWishesStatGlobal {
        uid: stat.uid,
        count_percentile: stat.count,
        luck_4_percentile: stat.luck_low,
        luck_5_percentile: stat.luck_high,
    })
    .collect()
}

async fn standard(pool: &PgPool) -> Result<()> {
    let stats = calculate_stats(database::gi::wishes_stats::standard::get_all(pool).await?);
    for batch in stats.chunks(UPDATE_BATCH_SIZE) {
        database::gi::wishes_stats_global::standard::set_bulk(batch, pool).await?;
    }
    Ok(())
}

async fn character(pool: &PgPool) -> Result<()> {
    let stats = calculate_stats(database::gi::wishes_stats::character::get_all(pool).await?);
    for batch in stats.chunks(UPDATE_BATCH_SIZE) {
        database::gi::wishes_stats_global::character::set_bulk(batch, pool).await?;
    }
    Ok(())
}

async fn weapon(pool: &PgPool) -> Result<()> {
    let stats = calculate_stats(database::gi::wishes_stats::weapon::get_all(pool).await?);
    for batch in stats.chunks(UPDATE_BATCH_SIZE) {
        database::gi::wishes_stats_global::weapon::set_bulk(batch, pool).await?;
    }
    Ok(())
}

async fn chronicled(pool: &PgPool) -> Result<()> {
    let stats = calculate_stats(database::gi::wishes_stats::chronicled::get_all(pool).await?);
    for batch in stats.chunks(UPDATE_BATCH_SIZE) {
        database::gi::wishes_stats_global::chronicled::set_bulk(batch, pool).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[actix_web::test]
    async fn aggregate_count_read_and_bulk_write_round_trip() {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();

        let suffix = (Uuid::new_v4().as_u128() % 10_000_000) as i32;
        let first_uid = 1_600_000_000 + suffix;
        let second_uid = first_uid + 10_000_000;
        let item = 1_800_000_000 + suffix;
        sqlx::query("INSERT INTO gi_profiles (uid, name) VALUES ($1, 'batch-a'), ($2, 'batch-b')")
            .bind(first_uid)
            .bind(second_uid)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO gi_characters (id, rarity) VALUES ($1, 5)")
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO gi_wishes_stats_standard (uid, luck_4, luck_5) VALUES ($1, 8, 70), ($2, 6, 80)",
        )
        .bind(first_uid)
        .bind(second_uid)
        .execute(&pool)
        .await
        .unwrap();

        for (uid, count) in [(first_uid, 101_i32), (second_uid, 100_i32)] {
            sqlx::query(
                "INSERT INTO gi_wishes_standard (id, uid, character, timestamp, official) SELECT $1::bigint + pull, $2, $3, to_timestamp(1700000000 + pull), true FROM generate_series(1, $4) AS pull",
            )
            .bind(i64::from(uid) * 1_000)
            .bind(uid)
            .bind(item)
            .bind(count)
            .execute(&pool)
            .await
            .unwrap();
        }

        let source = database::gi::wishes_stats::standard::get_all(&pool)
            .await
            .unwrap();
        assert_eq!(
            source
                .iter()
                .find(|stat| stat.uid == first_uid)
                .unwrap()
                .wish_count,
            Some(101)
        );
        assert_eq!(
            source
                .iter()
                .find(|stat| stat.uid == second_uid)
                .unwrap()
                .wish_count,
            Some(100)
        );

        standard(&pool).await.unwrap();
        let first = database::gi::wishes_stats_global::standard::get_by_uid(first_uid, &pool)
            .await
            .unwrap()
            .unwrap();
        let second = database::gi::wishes_stats_global::standard::get_by_uid(second_uid, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (
                first.count_percentile,
                first.luck_4_percentile,
                first.luck_5_percentile,
            ),
            (0.0, 0.5, 0.0)
        );
        assert_eq!(
            (
                second.count_percentile,
                second.luck_4_percentile,
                second.luck_5_percentile,
            ),
            (0.5, 0.0, 0.5)
        );

        sqlx::query("DELETE FROM gi_profiles WHERE uid = ANY($1::integer[])")
            .bind(vec![first_uid, second_uid])
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM gi_characters WHERE id = $1")
            .bind(item)
            .execute(&pool)
            .await
            .unwrap();
    }
}
