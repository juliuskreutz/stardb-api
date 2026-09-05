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
}

macro_rules! count_registry { ($( $variant:ident => $module:ident ),* $(,)?) => {
pub(crate) async fn get_all_by_pool(kind: crate::GachaType, pool:&sqlx::PgPool)->anyhow::Result<Vec<DbWarpsStatCount>> {match kind {$(crate::GachaType::$variant => $module::get_all(pool).await,)*_ => anyhow::bail!("pool has no calculated stats"),}}
};}
count_registry! {Standard => standard,Special => special,Lc => lc,Collab => collab,CollabLc => collab_lc}
