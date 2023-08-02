use std::time::Duration;

use actix_web::rt::{self, time};
use sqlx::PgPool;

use crate::{
    database::{self, DbConnection},
    mihomo, Result,
};

pub async fn verifications(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60 * 5));

        loop {
            interval.tick().await;

            let _ = update(&pool).await;
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

        let connection = DbConnection {
            uid: verification.uid,
            username: verification.username.clone(),
        };

        database::set_connection(&connection, pool).await?;
    }

    Ok(())
}
