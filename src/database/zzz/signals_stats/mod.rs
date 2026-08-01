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
