use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams},
    database,
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
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WishType {
    Character,
    Weapon,
}

impl From<database::gi::wishes::DbWish> for Wish {
    fn from(wish: database::gi::wishes::DbWish) -> Self {
        let r#type = if wish.character.is_some() {
            WishType::Character
        } else {
            WishType::Weapon
        };

        Self {
            r#type,
            id: wish.id.to_string(),
            name: wish.name.unwrap(),
            rarity: wish.rarity.unwrap(),
            item_id: wish.character.or(wish.weapon).unwrap(),
            timestamp: wish.timestamp,
            pull: 0,
            pull_4: 0,
            pull_5: 0,
        }
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
            if let Ok(connection) =
                database::gi::connections::get_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let name = database::gi::profiles::get_by_uid(uid, &pool).await?.name;

    let language = language_params.lang;

    // Beginner
    let mut beginner = Wishes::default();
    let mut beginner_pull = 0;
    let mut beginner_pull_4 = 0;
    let mut beginner_pull_5 = 0;

    for wish in database::gi::wishes::beginner::get_by_uid(uid, language, &pool).await? {
        let mut wish: Wish = wish.into();

        beginner_pull += 1;
        beginner_pull_4 += 1;
        beginner_pull_5 += 1;

        wish.pull = beginner_pull;
        wish.pull_4 = beginner_pull_4;
        wish.pull_5 = beginner_pull_5;

        match wish.rarity {
            4 => beginner_pull_4 = 0,
            5 => {
                beginner_pull_5 = 0;
            }
            _ => {}
        }

        beginner.wishes.push(wish);
    }

    beginner.pull_4 = beginner_pull_4;
    beginner.max_pull_4 = 10;
    beginner.probability_4 = if beginner_pull_4 < 9 { 9.4 } else { 100.0 };

    beginner.pull_5 = beginner_pull_5;
    beginner.max_pull_5 = 90;
    beginner.probability_5 = if beginner_pull_5 < 89 {
        0.6 + 6.0 * beginner_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    beginner.count = beginner.wishes.len();
    // Beginner

    // Standard
    let mut standard = Wishes::default();
    let mut standard_pull = 0;
    let mut standard_pull_4 = 0;
    let mut standard_pull_5 = 0;

    for wish in database::gi::wishes::standard::get_by_uid(uid, language, &pool).await? {
        let mut wish: Wish = wish.into();

        standard_pull += 1;
        standard_pull_4 += 1;
        standard_pull_5 += 1;

        wish.pull = standard_pull;
        wish.pull_4 = standard_pull_4;
        wish.pull_5 = standard_pull_5;

        match wish.rarity {
            4 => standard_pull_4 = 0,
            5 => {
                standard_pull_5 = 0;
            }
            _ => {}
        }

        standard.wishes.push(wish);
    }

    standard.pull_4 = standard_pull_4;
    standard.max_pull_4 = 10;
    standard.probability_4 = if standard_pull_4 < 9 { 9.4 } else { 100.0 };

    standard.pull_5 = standard_pull_5;
    standard.max_pull_5 = 90;
    standard.probability_5 = if standard_pull_5 < 89 {
        0.6 + 6.0 * standard_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    standard.count = standard.wishes.len();
    // Standard

    // Character
    let mut character = Wishes::default();
    let mut character_pull = 0;
    let mut character_pull_4 = 0;
    let mut character_pull_5 = 0;

    for wish in database::gi::wishes::character::get_by_uid(uid, language, &pool).await? {
        let mut wish: Wish = wish.into();

        character_pull += 1;
        character_pull_4 += 1;
        character_pull_5 += 1;

        wish.pull = character_pull;
        wish.pull_4 = character_pull_4;
        wish.pull_5 = character_pull_5;

        match wish.rarity {
            4 => character_pull_4 = 0,
            5 => {
                character_pull_5 = 0;
            }
            _ => {}
        }

        character.wishes.push(wish);
    }

    character.pull_4 = character_pull_4;
    character.max_pull_4 = 10;
    character.probability_4 = if character_pull_4 < 9 { 9.4 } else { 100.0 };

    character.pull_5 = character_pull_5;
    character.max_pull_5 = 90;
    character.probability_5 = if character_pull_5 < 89 {
        0.6 + 6.0 * character_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    character.count = character.wishes.len();
    // Character

    // Weapon
    let mut weapon = Wishes::default();
    let mut weapon_pull = 0;
    let mut weapon_pull_4 = 0;
    let mut weapon_pull_5 = 0;

    for wish in database::gi::wishes::weapon::get_by_uid(uid, language, &pool).await? {
        let mut wish: Wish = wish.into();

        weapon_pull += 1;
        weapon_pull_4 += 1;
        weapon_pull_5 += 1;

        wish.pull = weapon_pull;
        wish.pull_4 = weapon_pull_4;
        wish.pull_5 = weapon_pull_5;

        match wish.rarity {
            4 => weapon_pull_4 = 0,
            5 => {
                weapon_pull_5 = 0;
            }
            _ => {}
        }

        weapon.wishes.push(wish);
    }

    weapon.pull_4 = weapon_pull_4;
    weapon.max_pull_4 = 10;
    weapon.probability_4 = if weapon_pull_4 < 9 { 9.4 } else { 100.0 };

    weapon.pull_5 = weapon_pull_5;
    weapon.max_pull_5 = 90;
    weapon.probability_5 = if weapon_pull_5 < 89 {
        0.6 + 6.0 * weapon_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    weapon.count = weapon.wishes.len();
    // Weapon

    // Chronicled
    let mut chronicled = Wishes::default();
    let mut chronicled_pull = 0;
    let mut chronicled_pull_4 = 0;
    let mut chronicled_pull_5 = 0;

    for wish in database::gi::wishes::chronicled::get_by_uid(uid, language, &pool).await? {
        let mut wish: Wish = wish.into();

        chronicled_pull += 1;
        chronicled_pull_4 += 1;
        chronicled_pull_5 += 1;

        wish.pull = chronicled_pull;
        wish.pull_4 = chronicled_pull_4;
        wish.pull_5 = chronicled_pull_5;

        match wish.rarity {
            4 => chronicled_pull_4 = 0,
            5 => {
                chronicled_pull_5 = 0;
            }
            _ => {}
        }

        chronicled.wishes.push(wish);
    }

    chronicled.pull_4 = chronicled_pull_4;
    chronicled.max_pull_4 = 10;
    chronicled.probability_4 = if chronicled_pull_4 < 9 { 9.4 } else { 100.0 };

    chronicled.pull_5 = chronicled_pull_5;
    chronicled.max_pull_5 = 90;
    chronicled.probability_5 = if chronicled_pull_5 < 89 {
        0.6 + 6.0 * chronicled_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    chronicled.count = chronicled.wishes.len();
    // Chronicled

    if let Some(stats) = database::gi::wishes_stats::standard::get_by_uid(uid, &pool).await? {
        let global_stats = database::gi::wishes_stats_global::standard::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        standard.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats: None,
            global_stats,
        })
    }

    if let Some(stats) = database::gi::wishes_stats::character::get_by_uid(uid, &pool).await? {
        let win_stats = Some(WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        });

        let global_stats = database::gi::wishes_stats_global::character::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        character.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats,
            global_stats,
        })
    }

    if let Some(stats) = database::gi::wishes_stats::weapon::get_by_uid(uid, &pool).await? {
        let win_stats = Some(WinStats {
            win_rate: stats.win_rate,
            win_streak: stats.win_streak,
            loss_streak: stats.loss_streak,
        });

        let global_stats = database::gi::wishes_stats_global::weapon::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        weapon.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats,
            global_stats,
        })
    }

    if let Some(stats) = database::gi::wishes_stats::chronicled::get_by_uid(uid, &pool).await? {
        let global_stats = database::gi::wishes_stats_global::chronicled::get_by_uid(uid, &pool)
            .await?
            .map(|stats| GlobalStats {
                count_percentile: stats.count_percentile,
                luck_4_percentile: stats.luck_4_percentile,
                luck_5_percentile: stats.luck_5_percentile,
            });

        chronicled.stats = Some(Stats {
            luck_4: stats.luck_4,
            luck_5: stats.luck_5,
            win_stats: None,
            global_stats,
        })
    }

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
