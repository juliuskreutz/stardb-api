use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey};
use futures::lock::Mutex;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, Language};

#[derive(utoipa::OpenApi)]
#[openapi(
    tags((name = "users/me/export")),
    paths(get_export),
    components(schemas(
        Export,
        UserExport,
        Hsr,
        Zzz,
        Gi,
        HsrUid,
        ZzzUid,
        GiUid,
        Warps,
        Warp,
        WarpType,
        Signals,
        Signal,
        SignalType,
        Wishes,
        Wish,
        WishType,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_export);
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Export {
    user: UserExport,
    signature: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UserExport {
    username: String,
    hsr: Hsr,
    zzz: Zzz,
    gi: Gi,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Hsr {
    achievements: Vec<i32>,
    uids: Vec<HsrUid>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct HsrUid {
    uid: i32,
    verified: bool,
    private: bool,
    warps: Warps,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Warps {
    departure: Vec<Warp>,
    standard: Vec<Warp>,
    character: Vec<Warp>,
    light_cone: Vec<Warp>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Warp {
    id: String,
    item_id: i32,
    r#type: WarpType,
    timestamp: DateTime<Utc>,
    official: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum WarpType {
    Character,
    LightCone,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Zzz {
    achievements: Vec<i32>,
    uids: Vec<ZzzUid>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct ZzzUid {
    uid: i32,
    verified: bool,
    private: bool,
    signals: Signals,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Signals {
    standard: Vec<Signal>,
    character: Vec<Signal>,
    w_engine: Vec<Signal>,
    bangboo: Vec<Signal>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Signal {
    id: String,
    item_id: i32,
    r#type: SignalType,
    timestamp: DateTime<Utc>,
    official: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum SignalType {
    Character,
    WEngine,
    Bangboo,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Gi {
    achievements: Vec<i32>,
    uids: Vec<GiUid>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct GiUid {
    uid: i32,
    verified: bool,
    private: bool,
    wishes: Wishes,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Wishes {
    beginner: Vec<Wish>,
    standard: Vec<Wish>,
    character: Vec<Wish>,
    weapon: Vec<Wish>,
    chronicled: Vec<Wish>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Wish {
    id: String,
    item_id: i32,
    r#type: WishType,
    timestamp: DateTime<Utc>,
    official: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum WishType {
    Character,
    Weapon,
}

impl From<database::warps::DbWarp> for Warp {
    fn from(warp: database::warps::DbWarp) -> Self {
        let r#type = if warp.character.is_some() {
            WarpType::Character
        } else {
            WarpType::LightCone
        };

        Self {
            r#type,
            id: warp.id.to_string(),
            item_id: warp.character.or(warp.light_cone).unwrap(),
            timestamp: warp.timestamp,
            official: warp.official,
        }
    }
}

impl From<database::zzz::signals::DbSignal> for Signal {
    fn from(signal: database::zzz::signals::DbSignal) -> Self {
        let r#type = if signal.character.is_some() {
            SignalType::Character
        } else if signal.w_engine.is_some() {
            SignalType::WEngine
        } else {
            SignalType::Bangboo
        };

        Self {
            r#type,
            id: signal.id.to_string(),
            item_id: signal
                .character
                .or(signal.w_engine)
                .or(signal.bangboo)
                .unwrap(),
            timestamp: signal.timestamp,
            official: signal.official,
        }
    }
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
            item_id: wish.character.or(wish.weapon).unwrap(),
            timestamp: wish.timestamp,
            official: wish.official,
        }
    }
}

#[utoipa::path(
    tag = "users/me/export",
    get,
    path = "/api/users/me/export",
    responses(
        (status = 200, description = "Export", body = Export),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/export")]
async fn get_export(
    session: Session,
    signing_key: web::Data<Mutex<SigningKey>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let hsr = {
        let achievements =
            database::users_achievements_completed::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|a| a.id)
                .collect();

        let mut uids = Vec::new();

        for connection in database::connections::get_by_username(&username, &pool).await? {
            let uid = connection.uid;
            let verified = connection.verified;
            let private = connection.private;

            let departure = database::warps::departure::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Warp::from)
                .collect();

            let standard = database::warps::standard::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Warp::from)
                .collect();

            let character = database::warps::special::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Warp::from)
                .collect();

            let light_cone = database::warps::lc::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Warp::from)
                .collect();

            let warps = Warps {
                departure,
                standard,
                character,
                light_cone,
            };

            uids.push(HsrUid {
                uid,
                verified,
                private,
                warps,
            });
        }

        Hsr { achievements, uids }
    };

    let zzz = {
        let achievements =
            database::zzz::users_achievements_completed::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|a| a.id)
                .collect();

        let mut uids = Vec::new();

        for connection in database::zzz::connections::get_by_username(&username, &pool).await? {
            let uid = connection.uid;
            let verified = connection.verified;
            let private = connection.private;

            let standard = database::zzz::signals::standard::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Signal::from)
                .collect();

            let character = database::zzz::signals::special::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Signal::from)
                .collect();

            let w_engine = database::zzz::signals::w_engine::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Signal::from)
                .collect();

            let bangboo = database::zzz::signals::bangboo::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Signal::from)
                .collect();

            let signals = Signals {
                standard,
                character,
                w_engine,
                bangboo,
            };

            uids.push(ZzzUid {
                uid,
                verified,
                private,
                signals,
            });
        }

        Zzz { achievements, uids }
    };

    let gi = {
        let achievements =
            database::gi::users_achievements_completed::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|a| a.id)
                .collect();

        let mut uids = Vec::new();

        for connection in database::gi::connections::get_by_username(&username, &pool).await? {
            let uid = connection.uid;
            let verified = connection.verified;
            let private = connection.private;

            let beginner = database::gi::wishes::beginner::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Wish::from)
                .collect();

            let standard = database::gi::wishes::standard::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Wish::from)
                .collect();

            let character = database::gi::wishes::character::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Wish::from)
                .collect();

            let weapon = database::gi::wishes::weapon::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Wish::from)
                .collect();

            let chronicled = database::gi::wishes::chronicled::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Wish::from)
                .collect();

            let wishes = Wishes {
                beginner,
                standard,
                character,
                weapon,
                chronicled,
            };

            uids.push(GiUid {
                uid,
                verified,
                private,
                wishes,
            });
        }

        Gi { achievements, uids }
    };

    let user = UserExport {
        username,
        hsr,
        zzz,
        gi,
    };

    let user_bytes = serde_json::to_vec(&user)?;
    let signature = signing_key.lock().await.sign(&user_bytes).to_string();

    let export = Export { user, signature };

    Ok(HttpResponse::Ok().json(export))
}
