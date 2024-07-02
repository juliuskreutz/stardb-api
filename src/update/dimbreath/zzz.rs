use std::{
    path::Path,
    time::{Duration, Instant},
};

use actix_web::rt;
use async_process::Command;
use sqlx::PgPool;

pub async fn spawn(pool: PgPool) {
    actix::Arbiter::new().spawn(async move {
        let mut interval = rt::time::interval(Duration::from_secs(60 * 10));

        let mut up_to_date = false;

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&mut up_to_date, pool.clone()).await {
                error!(
                    "Dimbreath zzz update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                info!(
                    "Dimbreath zzz update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(up_to_date: &mut bool, pool: PgPool) -> anyhow::Result<()> {
    if !Path::new("ZenlessData").exists() {
        Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "https://github.com/Dimbreath/ZenlessData",
            ])
            .output()
            .await?;

        *up_to_date = false;
    }

    let output = String::from_utf8(
        Command::new("git")
            .arg("pull")
            .current_dir("ZenlessData")
            .output()
            .await?
            .stdout,
    )?;

    if !output.contains("Already up to date.") {
        *up_to_date = false;
    }

    if *up_to_date {
        return Ok(());
    }

    Ok(())
}
