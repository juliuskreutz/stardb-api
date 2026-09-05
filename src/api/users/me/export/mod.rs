//! Signed account exports containing achievements, connections, and every registered pull pool.
//! The signature covers serialized user data, including stored provenance flags.

use crate::gacha::imports::PullItem as StoredItem;
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

/// Returns the OpenAPI fragment for this module’s routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module’s HTTP routes with the application.
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
    collab: Vec<Warp>,
    collab_lc: Vec<Warp>,
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
    exclusive_rescreening: Vec<Signal>,
    w_engine_reverberation: Vec<Signal>,
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
    /// Maps stored item identity and provenance to the signed export’s wire fields.
    fn from(warp: database::warps::DbWarp) -> Self {
        let r#type = if matches!(warp.item, StoredItem::Character(_)) {
            WarpType::Character
        } else {
            WarpType::LightCone
        };

        Self {
            r#type,
            id: warp.id.to_string(),
            item_id: warp.item.id(),
            timestamp: warp.timestamp,
            official: warp.official,
        }
    }
}

impl From<database::zzz::signals::DbSignal> for Signal {
    /// Maps stored item identity and provenance to the signed export’s wire fields.
    fn from(signal: database::zzz::signals::DbSignal) -> Self {
        let r#type = if matches!(signal.item, StoredItem::Character(_)) {
            SignalType::Character
        } else if matches!(signal.item, StoredItem::WEngine(_)) {
            SignalType::WEngine
        } else {
            SignalType::Bangboo
        };

        Self {
            r#type,
            id: signal.id.to_string(),
            item_id: signal.item.id(),
            timestamp: signal.timestamp,
            official: signal.official,
        }
    }
}

impl From<database::gi::wishes::DbWish> for Wish {
    /// Maps stored item identity and provenance to the signed export’s wire fields.
    fn from(wish: database::gi::wishes::DbWish) -> Self {
        let r#type = if matches!(wish.item, StoredItem::Character(_)) {
            WishType::Character
        } else {
            WishType::Weapon
        };

        Self {
            r#type,
            id: wish.id.to_string(),
            item_id: wish.item.id(),
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
/// Exports the signed-in account’s achievements and pull histories, then signs the
/// serialized user payload. Returns 400 without a session username and propagates
/// database, row-validation, and serialization errors rather than returning a partial file.
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

            let collab = database::warps::collab::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();
            let collab_lc = database::warps::collab_lc::get_by_uid(uid, Language::En, &pool)
                .await?
                .into_iter()
                .map(Into::into)
                .collect();
            let warps = Warps {
                departure,
                standard,
                character,
                light_cone,
                collab,
                collab_lc,
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

            let exclusive_rescreening =
                database::zzz::signals::exclusive_rescreening::get_by_uid(uid, Language::En, &pool)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect();
            let w_engine_reverberation =
                database::zzz::signals::w_engine_reverberation::get_by_uid(
                    uid,
                    Language::En,
                    &pool,
                )
                .await?
                .into_iter()
                .map(Into::into)
                .collect();
            let signals = Signals {
                standard,
                character,
                w_engine,
                bangboo,
                exclusive_rescreening,
                w_engine_reverberation,
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

#[cfg(test)]
mod serialization_tests {
    use super::*;
    #[test]
    fn every_item_kind_preserves_export_wire_strings() {
        let row = database::warps::DbWarp {
            id: 42,
            item: StoredItem::Character(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Warp::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"character","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
        let row = database::warps::DbWarp {
            id: 42,
            item: StoredItem::LightCone(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Warp::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"light_cone","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
        let row = database::gi::wishes::DbWish {
            id: 42,
            item: StoredItem::Character(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Wish::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"character","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
        let row = database::gi::wishes::DbWish {
            id: 42,
            item: StoredItem::Weapon(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Wish::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"weapon","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
        let row = database::zzz::signals::DbSignal {
            id: 42,
            item: StoredItem::Character(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Signal::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"character","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
        let row = database::zzz::signals::DbSignal {
            id: 42,
            item: StoredItem::WEngine(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Signal::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"w_engine","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
        let row = database::zzz::signals::DbSignal {
            id: 42,
            item: StoredItem::Bangboo(1700000000),
            rarity: 4,
            name: Some("fixture".into()),
            timestamp: chrono::DateTime::from_timestamp(1700000000, 0).unwrap(),
            official: true,
        };
        let value = serde_json::to_value(Signal::from(row)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"id":"42","item_id":1700000000,"type":"bangboo","timestamp":"2023-11-14T22:13:20Z","official":true})
        );
    }
    #[test]
    fn signed_export_includes_collab_and_2026_pool_fields() {
        let timestamp = chrono::DateTime::from_timestamp(1700000000, 0).unwrap();
        let signal = || Signal {
            id: "42".into(),
            item_id: 1,
            r#type: SignalType::Character,
            timestamp,
            official: true,
        };
        let signals = Signals {
            standard: vec![],
            character: vec![],
            w_engine: vec![],
            bangboo: vec![],
            exclusive_rescreening: vec![signal()],
            w_engine_reverberation: vec![signal()],
        };
        let value = serde_json::to_value(signals).unwrap();
        assert_eq!(value["exclusive_rescreening"].as_array().unwrap().len(), 1);
        assert_eq!(value["w_engine_reverberation"].as_array().unwrap().len(), 1);
        let warp = || Warp {
            id: "42".into(),
            item_id: 1,
            r#type: WarpType::Character,
            timestamp,
            official: true,
        };
        let value = serde_json::to_value(Warps {
            departure: vec![],
            standard: vec![],
            character: vec![],
            light_cone: vec![],
            collab: vec![warp()],
            collab_lc: vec![warp()],
        })
        .unwrap();
        assert_eq!(value["collab"].as_array().unwrap().len(), 1);
        assert_eq!(value["collab_lc"].as_array().unwrap().len(), 1);
    }
}
