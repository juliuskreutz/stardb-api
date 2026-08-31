pub mod character;
pub mod chronicled;
pub mod standard;
pub mod weapon;

/// Shared write shape for Genshin population percentiles.
pub struct DbWishesStatGlobal {
    pub uid: i32,
    pub count_percentile: f64,
    pub luck_4_percentile: f64,
    pub luck_5_percentile: f64,
}
