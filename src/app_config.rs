use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_true")]
    pub enable_update_hsr_achievements_percent: bool,

    #[serde(default = "default_true")]
    pub enable_update_zzz_achievements_percent: bool,

    #[serde(default = "default_true")]
    pub enable_update_gi_achievements_percent: bool,

    #[serde(default = "default_true")]
    pub enable_update_dimbreath_hsr: bool,

    #[serde(default = "default_true")]
    pub enable_update_dimbreath_zzz: bool,

    #[serde(default = "default_true")]
    pub enable_update_dimbreath_gi: bool,

    #[serde(default = "default_true")]
    pub enable_update_star_rail_res: bool,

    #[serde(default = "default_true")]
    pub enable_update_scores: bool,

    #[serde(default = "default_true")]
    pub enable_update_achievement_trackers: bool,

    #[serde(default = "default_true")]
    pub enable_update_warps_stats: bool,

    #[serde(default = "default_true")]
    pub enable_update_signals_stats: bool,

    #[serde(default = "default_true")]
    pub enable_update_wishes_stats: bool,

    #[serde(default = "default_true")]
    pub enable_update_sitemaps: bool,
}

fn default_true() -> bool {
    true
}
