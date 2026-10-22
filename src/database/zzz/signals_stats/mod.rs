//! ZZZ per-UID pity and win-stat persistence; count reads feed population ranking.

pub mod bangboo;
pub mod exclusive_rescreening;
pub mod special;
pub mod standard;
pub mod w_engine;
pub mod w_engine_reverberation;

/// Per-user pity values joined with their stored signal count.
pub struct DbSignalsStatCount {
    pub uid: i32,
    pub luck_a: f64,
    pub luck_s: f64,
    pub signal_count: Option<i64>,
}

macro_rules! count_registry { ($( $variant:ident => $module:ident ),* $(,)?) => {
/// Reads local stats joined with per-pool pull counts for population ranking.
/// Eligibility is applied by the updater; this read does not filter short histories. Result order is unspecified.
pub(crate) async fn get_all_by_pool(kind: crate::ZzzGachaType, pool:&sqlx::PgPool)->anyhow::Result<Vec<DbSignalsStatCount>> {match kind {$(crate::ZzzGachaType::$variant => $module::get_all(pool).await,)*}}
};}
count_registry! {Standard => standard,Special => special,WEngine => w_engine,Bangboo => bangboo,ExclusiveRescreening => exclusive_rescreening,WEngineReverberation => w_engine_reverberation}
