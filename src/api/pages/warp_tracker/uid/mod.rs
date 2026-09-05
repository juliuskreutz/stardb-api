use crate::gacha::imports::PullItem as StoredItem;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams},
    database,
    gacha::{
        banner::{BannerCatalog, GuaranteeState, GuaranteedOutcome},
        imports::{PullItem, PullPool},
    },
    GachaType,
};

#[derive(OpenApi)]
#[openapi(paths(get_warp_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warp_tracker);
}

#[derive(Serialize)]
struct Warp {
    r#type: WarpType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    pull: usize,
    pull_4: usize,
    pull_5: usize,
    timestamp: DateTime<Utc>,
    win: Option<WinType>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WinType {
    Win,
    Loss,
    Guarantee,
}

impl TryFrom<database::warps::DbWarp> for Warp {
    type Error = anyhow::Error;
    fn try_from(warp: database::warps::DbWarp) -> anyhow::Result<Self> {
        let r#type = if matches!(warp.item, StoredItem::Character(_)) {
            WarpType::Character
        } else {
            WarpType::LightCone
        };

        Ok(Self {
            r#type,
            id: warp.id.to_string(),
            name: warp
                .name
                .ok_or_else(|| anyhow::anyhow!("missing localized pull name"))?,
            rarity: warp.rarity,
            item_id: warp.item.id(),
            timestamp: warp.timestamp,
            pull: 0,
            pull_4: 0,
            pull_5: 0,
            win: None,
        })
    }
}

#[derive(Serialize)]
struct WarpTracker {
    standard: Warps,
    departure: Warps,
    special: Warps,
    lc: Warps,
    collab: Warps,
    collab_lc: Warps,
    name: String,
}

#[derive(Default, Serialize)]
struct Warps {
    warps: Vec<Warp>,
    probability_4: f64,
    probability_5: f64,
    pull_4: usize,
    pull_5: usize,
    max_pull_4: usize,
    max_pull_5: usize,
    count: usize,
    stats: Option<Stats>,
}

#[derive(Serialize)]
struct Stats {
    luck_4: f64,
    luck_5: f64,
    win_stats: Option<WinStats>,
    global_stats: Option<GlobalStats>,
}

#[derive(Serialize)]
struct WinStats {
    win_rate: f64,
    win_streak: i32,
    loss_streak: i32,
}

#[derive(Serialize)]
struct GlobalStats {
    count_percentile: f64,
    luck_4_percentile: f64,
    luck_5_percentile: f64,
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/warp-tracker/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "WarpTracker"),
    )
)]
#[get("/api/pages/warp-tracker/{uid}", guard = "private")]
async fn get_warp_tracker(
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if database::admins::exists(&username, &pool).await? {
                forbidden = false;
            } else if let Ok(connection) =
                database::connections::get_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let language = language_params.lang;

    let Some(mihomo) = database::mihomo::get_one_by_uid(uid, &pool).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let name = mihomo.name;

    let banner_catalog = BannerCatalog::from_hsr(database::banners::get_all(&pool).await?);

    // region Departure
    let mut departure = build_set(
        database::warps::departure::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GachaType::Departure,
        &banner_catalog,
    );
    // endregion Departure

    // region Standard
    let mut standard = build_set(
        database::warps::standard::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GachaType::Standard,
        &banner_catalog,
    );
    // endregion Standard

    // region Special
    let mut special = build_set(
        database::warps::special::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GachaType::Special,
        &banner_catalog,
    );
    // endregion Special

    // region Lc
    let mut lc = build_set(
        database::warps::lc::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GachaType::Lc,
        &banner_catalog,
    );
    // endregion Lc

    // region Collab
    let mut collab = build_set(
        database::warps::collab::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GachaType::Collab,
        &banner_catalog,
    );
    // endregion Collab

    // region Collab LC
    let mut collab_lc = build_set(
        database::warps::collab_lc::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GachaType::CollabLc,
        &banner_catalog,
    );
    // endregion Collab LC

    // region Stats
    set_stats(&mut standard, GachaType::Standard, uid, &pool).await?;

    set_stats(&mut special, GachaType::Special, uid, &pool).await?;

    set_stats(&mut lc, GachaType::Lc, uid, &pool).await?;

    set_stats(&mut collab, GachaType::Collab, uid, &pool).await?;

    set_stats(&mut collab_lc, GachaType::CollabLc, uid, &pool).await?;
    // endregion Stats

    let warp_tracker = WarpTracker {
        standard,
        departure,
        special,
        lc,
        collab,
        collab_lc,
        name,
    };

    Ok(HttpResponse::Ok().json(warp_tracker))
}

/// Pool parameters keep historical caps explicit; None preserves Departure's zero summaries.
#[derive(Clone, Copy)]
struct Pity {
    base_4: f64,
    base_5: f64,
    gain_5: f64,
    soft_start_5: usize,
    hard_4: usize,
    hard_5: usize,
    max_4: usize,
    max_5: usize,
}
impl Pity {
    fn probabilities(self, low: usize, high: usize) -> (f64, f64) {
        (
            if low < self.hard_4 {
                self.base_4
            } else {
                100.0
            },
            if high < self.hard_5 {
                self.base_5 + self.gain_5 * high.saturating_sub(self.soft_start_5) as f64
            } else {
                100.0
            },
        )
    }
}
fn build_set(rows: Vec<Warp>, kind: GachaType, catalog: &BannerCatalog) -> Warps {
    let pity = match kind {
        GachaType::Departure => None,
        GachaType::Standard => Some(Pity {
            base_4: 5.1,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GachaType::Special => Some(Pity {
            base_4: 5.1,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GachaType::Lc => Some(Pity {
            base_4: 6.6,
            base_5: 0.8,
            gain_5: 7.0,
            soft_start_5: 64,
            hard_4: 9,
            hard_5: 79,
            max_4: 10,
            max_5: 80,
        }),
        GachaType::Collab => Some(Pity {
            base_4: 5.1,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GachaType::CollabLc => Some(Pity {
            base_4: 6.6,
            base_5: 0.8,
            gain_5: 7.0,
            soft_start_5: 64,
            hard_4: 9,
            hard_5: 79,
            max_4: 10,
            max_5: 80,
        }),
    };
    let mut result = Warps::default();
    let (mut pull_4, mut pull_5) = (0, 0);
    let mut guarantee = GuaranteeState::default();
    for (index, mut row) in rows.into_iter().enumerate() {
        pull_4 += 1;
        pull_5 += 1;
        row.pull = index + 1;
        row.pull_4 = pull_4;
        row.pull_5 = pull_5;
        if row.rarity == 4 {
            pull_4 = 0;
        }
        if row.rarity == 5 {
            pull_5 = 0;
            let item = match kind {
                GachaType::Departure => None,
                GachaType::Standard => None,
                GachaType::Special => Some(PullItem::Character(row.item_id)),
                GachaType::Lc => Some(PullItem::LightCone(row.item_id)),
                GachaType::Collab => Some(PullItem::Character(row.item_id)),
                GachaType::CollabLc => Some(PullItem::LightCone(row.item_id)),
            };
            if let Some(item) = item {
                row.win = Some(
                    match guarantee.advance(catalog.classify(
                        PullPool::Hsr(kind),
                        item,
                        row.timestamp,
                    )) {
                        GuaranteedOutcome::Win => WinType::Win,
                        GuaranteedOutcome::Loss => WinType::Loss,
                        GuaranteedOutcome::GuaranteedWin => WinType::Guarantee,
                    },
                );
            }
        }
        result.warps.push(row);
    }
    if let Some(pity) = pity {
        result.pull_4 = pull_4;
        result.pull_5 = pull_5;
        result.max_pull_4 = pity.max_4;
        result.max_pull_5 = pity.max_5;
        (result.probability_4, result.probability_5) = pity.probabilities(pull_4, pull_5);
    }
    result.count = result.warps.len();
    result
}
async fn set_stats(
    set: &mut Warps,
    kind: GachaType,
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<()> {
    macro_rules! load {
        ($module:ident, $win:ident) => {
            if let Some(stats) = database::warps_stats::$module::get_by_uid(uid, pool).await? {
                let global_stats = database::warps_stats_global::$module::get_by_uid(uid, pool)
                    .await?
                    .map(|stats| GlobalStats {
                        count_percentile: stats.count_percentile,
                        luck_4_percentile: stats.luck_4_percentile,
                        luck_5_percentile: stats.luck_5_percentile,
                    });
                let value = Stats {
                    luck_4: stats.luck_4,
                    luck_5: stats.luck_5,
                    win_stats: win!(stats, $win),
                    global_stats,
                };
                set.stats = Some(value);
            }
        };
    }
    macro_rules! win {
        ($stats:ident, yes) => {
            Some(WinStats {
                win_rate: $stats.win_rate,
                win_streak: $stats.win_streak,
                loss_streak: $stats.loss_streak,
            })
        };
        ($stats:ident, no) => {
            None
        };
    }
    match kind {
        GachaType::CollabLc => load!(collab_lc, yes),
        GachaType::Collab => load!(collab, yes),
        GachaType::Lc => load!(lc, yes),
        GachaType::Special => load!(special, yes),
        GachaType::Standard => load!(standard, no),
        GachaType::Departure => {}
    }
    Ok(())
}

#[cfg(test)]
mod baseline;
#[cfg(test)]
mod golden_tests;
