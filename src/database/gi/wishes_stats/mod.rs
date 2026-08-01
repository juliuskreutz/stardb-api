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
