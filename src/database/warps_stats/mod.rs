//! HSR per-UID pity and win-stat persistence; count reads feed population ranking.

pub mod collab;
pub mod collab_lc;
pub mod lc;
pub mod special;
pub mod standard;

pub struct DbWarpsStatCount {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
    pub warp_count: Option<i64>,
}

pub struct DbWarpsStat {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
    /// Rows in this pool when the stats were last calculated; NULL until backfilled.
    pub pull_count: Option<i32>,
}

macro_rules! count_registry { ($( $variant:ident => $module:ident ),* $(,)?) => {
/// Reads local stats with their stored pull counts for population ranking, after
/// backfilling counts that predate the column. SQL limits the population to UIDs with
/// at least 100 pulls. Result order is unspecified.
/// A pool without local-stat storage is rejected rather than silently returning no rows.
pub(crate) async fn get_all_by_pool(kind: crate::GachaType, pool:&sqlx::PgPool)->anyhow::Result<Vec<DbWarpsStatCount>> {match kind {$(crate::GachaType::$variant => {
    crate::database::stats_pull_counts::backfill(concat!("warps_stats_", stringify!($module)), concat!("warps_", stringify!($module)), pool).await?;
    $module::get_all(pool).await
},)*_ => anyhow::bail!("pool has no calculated stats"),}}
};}
count_registry! {Standard => standard,Special => special,Lc => lc,Collab => collab,CollabLc => collab_lc}
