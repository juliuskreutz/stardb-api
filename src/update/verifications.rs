use std::time::{Duration, Instant};

use anyhow::Result;
use sqlx::PgPool;

use crate::{database, mihomo};

pub async fn verifications(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 5));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                error!(
                    "Verifications update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                info!(
                    "Verifications update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let verifications = database::get_verifications(pool).await?;

    for verification in verifications {
        let api_data = mihomo::get(verification.uid).await?;

        if !api_data.player.signature.ends_with(&verification.token) {
            continue;
        }

        database::delete_verification_by_uid(verification.uid, pool).await?;

        let connection = database::DbConnection {
            uid: verification.uid,
            username: verification.username.clone(),
        };

        database::set_connection(&connection, pool).await?;
    }

    Ok(())
}
