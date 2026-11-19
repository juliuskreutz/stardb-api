//! Background refresh entry points and a shared driver with completion-based retry delays.

pub mod achievements_percent;
pub mod dimbreath;
pub mod gi_achievements_percent;
pub mod scores;
pub mod signals_stats;
pub mod star_rail_res;
pub mod warps_stats;
pub mod wishes_stats;
pub mod zzz_achievements_percent;

/// Run immediately, then wait after each completion. Failures always back off.
/// Delays start when the job finishes, so slow jobs never accumulate missed ticks or
/// launch overlapping refreshes. A transient failure uses the shorter retry delay
/// instead of either hammering the database or leaving the cache stale for an hour.
pub fn spawn_periodic<F, Fut>(
    label: &'static str,
    interval: std::time::Duration,
    retry: std::time::Duration,
    mut job: F,
) where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    actix::Arbiter::new().spawn(async move {
        loop {
            let start = std::time::Instant::now();
            let delay = match job().await {
                Ok(()) => {
                    info!(
                        "{label} update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                    interval
                }
                Err(e) => {
                    error!(
                        "{label} update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                    retry
                }
            };
            actix_web::rt::time::sleep(delay).await;
        }
    });
}
