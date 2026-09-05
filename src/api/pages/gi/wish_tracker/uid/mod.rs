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
    GiGachaType,
};

#[derive(OpenApi)]
#[openapi(paths(get_wish_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_wish_tracker);
}

#[derive(Serialize)]
struct Wish {
    r#type: WishType,
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
enum WishType {
    Character,
    Weapon,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WinType {
    Win,
    Loss,
    Guarantee,
}

impl TryFrom<database::gi::wishes::DbWish> for Wish {
    type Error = anyhow::Error;
    fn try_from(wish: database::gi::wishes::DbWish) -> anyhow::Result<Self> {
        let r#type = if matches!(wish.item, StoredItem::Character(_)) {
            WishType::Character
        } else {
            WishType::Weapon
        };

        Ok(Self {
            r#type,
            id: wish.id.to_string(),
            name: wish
                .name
                .ok_or_else(|| anyhow::anyhow!("missing localized pull name"))?,
            rarity: wish.rarity,
            item_id: wish.item.id(),
            timestamp: wish.timestamp,
            pull: 0,
            pull_4: 0,
            pull_5: 0,
            win: None,
        })
    }
}

#[derive(Serialize)]
struct WishTracker {
    name: String,
    beginner: Wishes,
    standard: Wishes,
    character: Wishes,
    weapon: Wishes,
    chronicled: Wishes,
}

#[derive(Default, Serialize)]
struct Wishes {
    wishes: Vec<Wish>,
    probability_4: f64,
    probability_5: f64,
    pull_4: usize,
    pull_5: usize,
    max_pull_4: usize,
    max_pull_5: usize,
    count: usize,
    stats: Option<Stats>,
}

#[derive(Default, Serialize)]
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
    tag = "pages/gi",
    get,
    path = "/api/pages/gi/wish-tracker/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "WishTracker"),
    )
)]
#[get("/api/pages/gi/wish-tracker/{uid}", guard = "private")]
async fn get_wish_tracker(
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::gi::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if database::admins::exists(&username, &pool).await? {
                forbidden = false;
            } else if let Ok(connection) =
                database::gi::connections::get_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let language = language_params.lang;

    let Some(profile) = database::gi::profiles::get_by_uid(uid, &pool).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let name = profile.name;

    let banner_catalog = BannerCatalog::from_gi(database::gi::banners::get_all(&pool).await?);

    // Beginner
    let beginner = build_set(
        database::gi::wishes::beginner::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GiGachaType::Beginner,
        &banner_catalog,
    );
    // Beginner

    // Standard
    let mut standard = build_set(
        database::gi::wishes::standard::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GiGachaType::Standard,
        &banner_catalog,
    );
    // Standard

    // Character
    let mut character = build_set(
        database::gi::wishes::character::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GiGachaType::Character,
        &banner_catalog,
    );
    // Character

    // Weapon
    let mut weapon = build_set(
        database::gi::wishes::weapon::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GiGachaType::Weapon,
        &banner_catalog,
    );
    // Weapon

    // Chronicled
    let mut chronicled = build_set(
        database::gi::wishes::chronicled::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        GiGachaType::Chronicled,
        &banner_catalog,
    );
    // Chronicled

    set_stats(&mut standard, GiGachaType::Standard, uid, &pool).await?;

    set_stats(&mut character, GiGachaType::Character, uid, &pool).await?;

    set_stats(&mut weapon, GiGachaType::Weapon, uid, &pool).await?;

    set_stats(&mut chronicled, GiGachaType::Chronicled, uid, &pool).await?;

    let wish_tracker = WishTracker {
        name,
        beginner,
        standard,
        character,
        weapon,
        chronicled,
    };

    Ok(HttpResponse::Ok().json(wish_tracker))
}

/// Pool-specific next-pull rates and displayed pity caps, in percentage units.
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
    // Counters are completed pulls since the last reset. The threshold therefore
    // describes the next pull (for example, 89 elapsed pulls imply 100% at pull 90).
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
/// Annotates chronological history separately from aggregate luck calculations.
fn build_set(rows: Vec<Wish>, kind: GiGachaType, catalog: &BannerCatalog) -> Wishes {
    let pity = match kind {
        GiGachaType::Beginner => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GiGachaType::Standard => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GiGachaType::Character => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GiGachaType::Weapon => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        GiGachaType::Chronicled => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
    };
    let mut result = Wishes::default();
    let (mut pull_4, mut pull_5) = (0, 0);
    let mut guarantee = GuaranteeState::default();
    for (index, mut row) in rows.into_iter().enumerate() {
        pull_4 += 1;
        pull_5 += 1;
        // The winning row displays the interval it completed, so annotate before
        // resetting counters for the next row.
        row.pull = index + 1;
        row.pull_4 = pull_4;
        row.pull_5 = pull_5;
        if row.rarity == 4 {
            pull_4 = 0;
        }
        // A five-star pull does not reset the four-star counter in this game.
        if row.rarity == 5 {
            pull_5 = 0;
            let item = match kind {
                GiGachaType::Beginner => None,
                GiGachaType::Standard => None,
                GiGachaType::Character => Some(PullItem::Character(row.item_id)),
                GiGachaType::Weapon => Some(PullItem::Weapon(row.item_id)),
                // Chronicled keeps its existing pity-only output; no guarantee model.
                GiGachaType::Chronicled => None,
            };
            if let Some(item) = item {
                row.win = Some(
                    match guarantee.advance(catalog.classify(
                        PullPool::Gi(kind),
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
        result.wishes.push(row);
    }
    if let Some(pity) = pity {
        result.pull_4 = pull_4;
        result.pull_5 = pull_5;
        result.max_pull_4 = pity.max_4;
        result.max_pull_5 = pity.max_5;
        (result.probability_4, result.probability_5) = pity.probabilities(pull_4, pull_5);
    }
    result.count = result.wishes.len();
    result
}
async fn set_stats(
    set: &mut Wishes,
    kind: GiGachaType,
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<()> {
    macro_rules! load {
        ($module:ident, $win:ident) => {
            if let Some(stats) = database::gi::wishes_stats::$module::get_by_uid(uid, pool).await? {
                let global_stats =
                    database::gi::wishes_stats_global::$module::get_by_uid(uid, pool)
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
        GiGachaType::Chronicled => load!(chronicled, no),
        GiGachaType::Weapon => load!(weapon, yes),
        GiGachaType::Character => load!(character, yes),
        GiGachaType::Standard => load!(standard, no),
        GiGachaType::Beginner => {}
    }
    Ok(())
}

#[cfg(test)]
mod baseline;
#[cfg(test)]
mod golden_tests;
