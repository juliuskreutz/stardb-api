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
    ZzzGachaType,
};

#[derive(OpenApi)]
#[openapi(paths(get_signal_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_signal_tracker);
}

#[derive(Serialize)]
struct Signal {
    r#type: SignalType,
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
enum SignalType {
    Agent,
    WEngine,
    Bangboo,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WinType {
    Win,
    Loss,
    Guarantee,
}

impl TryFrom<database::zzz::signals::DbSignal> for Signal {
    type Error = anyhow::Error;
    fn try_from(signal: database::zzz::signals::DbSignal) -> anyhow::Result<Self> {
        let r#type = if matches!(signal.item, StoredItem::Character(_)) {
            SignalType::Agent
        } else if matches!(signal.item, StoredItem::WEngine(_)) {
            SignalType::WEngine
        } else {
            SignalType::Bangboo
        };

        Ok(Self {
            r#type,
            id: signal.id.to_string(),
            name: signal
                .name
                .ok_or_else(|| anyhow::anyhow!("missing localized pull name"))?,
            rarity: signal.rarity,
            item_id: signal.item.id(),
            timestamp: signal.timestamp,
            pull: 0,
            pull_4: 0,
            pull_5: 0,
            win: None,
        })
    }
}

#[derive(Serialize)]
struct SignalTracker {
    standard: Signals,
    special: Signals,
    w_engine: Signals,
    bangboo: Signals,
    exclusive_rescreening: Signals,
    w_engine_reverberation: Signals,
}

#[derive(Default, Serialize)]
struct Signals {
    signals: Vec<Signal>,
    probability_4: f64,
    probability_5: f64,
    pull_4: usize,
    pull_5: usize,
    max_pull_4: usize,
    max_pull_5: usize,
    count: usize,
    stats: Stats,
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
    tag = "pages/zzz",
    get,
    path = "/api/pages/zzz/signal-tracker/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "SignalTracker"),
    )
)]
#[get("/api/pages/zzz/signal-tracker/{uid}", guard = "private")]
async fn get_signal_tracker(
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::zzz::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if database::admins::exists(&username, &pool).await? {
                forbidden = false;
            } else if let Ok(connection) =
                database::zzz::connections::get_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let language = language_params.lang;
    let banner_catalog = BannerCatalog::from_zzz(database::zzz::banners::get_all(&pool).await?);

    // Standard
    let mut standard = build_set(
        database::zzz::signals::standard::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        ZzzGachaType::Standard,
        &banner_catalog,
    );
    // Standard

    // Special
    let mut special = build_set(
        database::zzz::signals::special::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        ZzzGachaType::Special,
        &banner_catalog,
    );
    // Special

    // WEngine
    let mut w_engine = build_set(
        database::zzz::signals::w_engine::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        ZzzGachaType::WEngine,
        &banner_catalog,
    );
    // WEngine

    // Bangboo
    let mut bangboo = build_set(
        database::zzz::signals::bangboo::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        ZzzGachaType::Bangboo,
        &banner_catalog,
    );
    // Bangboo

    // Exclusive Rescreening
    let mut exclusive_rescreening = build_set(
        database::zzz::signals::exclusive_rescreening::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        ZzzGachaType::ExclusiveRescreening,
        &banner_catalog,
    );
    // Exclusive Rescreening

    // WEngine Reverberation
    let mut w_engine_reverberation = build_set(
        database::zzz::signals::w_engine_reverberation::get_by_uid(uid, language, &pool)
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect::<anyhow::Result<Vec<_>>>()?,
        ZzzGachaType::WEngineReverberation,
        &banner_catalog,
    );
    // WEngine Reverberation

    set_stats(&mut standard, ZzzGachaType::Standard, uid, &pool).await?;

    set_stats(&mut special, ZzzGachaType::Special, uid, &pool).await?;

    set_stats(&mut w_engine, ZzzGachaType::WEngine, uid, &pool).await?;

    set_stats(&mut bangboo, ZzzGachaType::Bangboo, uid, &pool).await?;

    set_stats(
        &mut exclusive_rescreening,
        ZzzGachaType::ExclusiveRescreening,
        uid,
        &pool,
    )
    .await?;

    set_stats(
        &mut w_engine_reverberation,
        ZzzGachaType::WEngineReverberation,
        uid,
        &pool,
    )
    .await?;

    let signal_tracker = SignalTracker {
        standard,
        special,
        w_engine,
        bangboo,
        exclusive_rescreening,
        w_engine_reverberation,
    };

    Ok(HttpResponse::Ok().json(signal_tracker))
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
fn build_set(rows: Vec<Signal>, kind: ZzzGachaType, catalog: &BannerCatalog) -> Signals {
    let pity = match kind {
        ZzzGachaType::Standard => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        ZzzGachaType::Special => Some(Pity {
            base_4: 9.4,
            base_5: 0.6,
            gain_5: 6.0,
            soft_start_5: 72,
            hard_4: 9,
            hard_5: 89,
            max_4: 10,
            max_5: 90,
        }),
        ZzzGachaType::WEngine => Some(Pity {
            base_4: 15.0,
            base_5: 1.0,
            gain_5: 7.0,
            soft_start_5: 64,
            hard_4: 9,
            hard_5: 79,
            max_4: 10,
            max_5: 80,
        }),
        ZzzGachaType::Bangboo => Some(Pity {
            base_4: 15.0,
            base_5: 1.0,
            gain_5: 7.0,
            soft_start_5: 64,
            hard_4: 9,
            hard_5: 79,
            max_4: 10,
            max_5: 80,
        }),
        ZzzGachaType::ExclusiveRescreening => None,
        ZzzGachaType::WEngineReverberation => None,
    };
    let mut result = Signals::default();
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
        if row.rarity == 3 {
            pull_4 = 0;
        }
        // An S-rank pull resets both displayed pity counters in ZZZ. HSR/GI and
        // aggregate ZZZ luck calculations intentionally use independent resets.
        if row.rarity == 4 {
            pull_5 = 0;
            pull_4 = 0;
            let item = match kind {
                ZzzGachaType::Standard => None,
                ZzzGachaType::Special => Some(PullItem::Character(row.item_id)),
                ZzzGachaType::WEngine => Some(PullItem::WEngine(row.item_id)),
                ZzzGachaType::Bangboo => None,
                ZzzGachaType::ExclusiveRescreening => Some(PullItem::Character(row.item_id)),
                ZzzGachaType::WEngineReverberation => Some(PullItem::WEngine(row.item_id)),
            };
            if let Some(item) = item {
                row.win = Some(
                    match guarantee.advance(catalog.classify(
                        PullPool::Zzz(kind),
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
        result.signals.push(row);
    }
    if let Some(pity) = pity {
        result.pull_4 = pull_4;
        result.pull_5 = pull_5;
        result.max_pull_4 = pity.max_4;
        result.max_pull_5 = pity.max_5;
        (result.probability_4, result.probability_5) = pity.probabilities(pull_4, pull_5);
    }
    result.count = result.signals.len();
    result
}
async fn set_stats(
    set: &mut Signals,
    kind: ZzzGachaType,
    uid: i32,
    pool: &PgPool,
) -> anyhow::Result<()> {
    macro_rules! load {
        ($module:ident, $win:ident) => {
            if let Some(stats) =
                database::zzz::signals_stats::$module::get_by_uid(uid, pool).await?
            {
                let global_stats =
                    database::zzz::signals_stats_global::$module::get_by_uid(uid, pool)
                        .await?
                        .map(|stats| GlobalStats {
                            count_percentile: stats.count_percentile,
                            luck_4_percentile: stats.luck_a_percentile,
                            luck_5_percentile: stats.luck_s_percentile,
                        });
                let value = Stats {
                    luck_4: stats.luck_a,
                    luck_5: stats.luck_s,
                    win_stats: win!(stats, $win),
                    global_stats,
                };
                set.stats = value;
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
        ZzzGachaType::WEngineReverberation => load!(w_engine_reverberation, yes),
        ZzzGachaType::ExclusiveRescreening => load!(exclusive_rescreening, yes),
        ZzzGachaType::Bangboo => load!(bangboo, no),
        ZzzGachaType::WEngine => load!(w_engine, yes),
        ZzzGachaType::Special => load!(special, yes),
        ZzzGachaType::Standard => load!(standard, no),
    }
    Ok(())
}

#[cfg(test)]
mod baseline;
#[cfg(test)]
mod golden_tests;
