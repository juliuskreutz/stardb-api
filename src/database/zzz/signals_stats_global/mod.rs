//! ZZZ population-percentile persistence and pool-specific routing.

pub mod bangboo;
pub mod exclusive_rescreening;
pub mod special;
pub mod standard;
pub mod w_engine;
pub mod w_engine_reverberation;

/// Shared write shape for ZZZ population percentiles.
pub struct DbSignalsStatGlobal {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_a_percentile: f64,
    pub luck_s_percentile: f64,
}

macro_rules! global_registry { ($( $variant:ident => $module:ident ),* $(,)?) => {
/// Upserts the supplied percentile rows in one pool; an empty slice is a no-op.
/// Only listed UIDs are changed, and database failures propagate.
pub(crate) async fn set_bulk_by_pool(kind:crate::ZzzGachaType,rows:&[DbSignalsStatGlobal],pool:&sqlx::PgPool)->anyhow::Result<()> { match kind {$(crate::ZzzGachaType::$variant => $module::set_bulk(rows,pool).await,)*}}
/// Deletes percentile rows for the supplied ineligible UIDs; an empty slice is a no-op.
/// This does not delete pull history or local stats.
pub(crate) async fn delete_bulk_by_pool(kind:crate::ZzzGachaType,uids:&[i32],pool:&sqlx::PgPool)->anyhow::Result<()> {match kind {$(crate::ZzzGachaType::$variant => $module::delete_bulk(uids,pool).await,)*}}
};}
global_registry! {Standard => standard,Special => special,WEngine => w_engine,Bangboo => bangboo,ExclusiveRescreening => exclusive_rescreening,WEngineReverberation => w_engine_reverberation}
