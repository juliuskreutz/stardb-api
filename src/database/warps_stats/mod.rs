pub mod lc;
pub mod special;
pub mod standard;
pub mod collab;
pub mod collab_lc;

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
}