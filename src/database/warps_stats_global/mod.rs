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
