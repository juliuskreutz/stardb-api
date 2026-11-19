//! Genshin per-UID pity and win-stat persistence; count reads feed population ranking.

pub mod character;
pub mod chronicled;
pub mod standard;
pub mod weapon;

/// Per-user pity values joined with their stored wish count.
pub struct DbWishesStatCount {
    pub uid: i32,
    pub luck_4: f64,
    pub luck_5: f64,
    pub wish_count: Option<i64>,
}

macro_rules! count_registry { ($( $variant:ident => $module:ident ),* $(,)?) => {
/// Reads local stats joined with per-pool pull counts for population ranking.
/// SQL limits the population to UIDs with at least 100 pulls. Result order is unspecified.
/// A pool without local-stat storage is rejected rather than silently returning no rows.
pub(crate) async fn get_all_by_pool(kind: crate::GiGachaType, pool:&sqlx::PgPool)->anyhow::Result<Vec<DbWishesStatCount>> {match kind {$(crate::GiGachaType::$variant => $module::get_all(pool).await,)*_ => anyhow::bail!("pool has no calculated stats"),}}
};}
count_registry! {Standard => standard,Character => character,Weapon => weapon,Chronicled => chronicled}
