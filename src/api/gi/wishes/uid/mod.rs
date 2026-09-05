use crate::gacha::imports::PullItem as StoredItem;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{gacha_history_forbidden, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/wishes/{uid}")),
    paths(get_gi_wishes),
    components(schemas(Wishes, Wish, WishType))
)]
struct ApiDoc;

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
    r#type: WishType,
    id: String,
    name: String,
    rarity: i32,
    item_id: i32,
    timestamp: DateTime<Utc>,
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
        })
    }
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
enum WishType {
    Character,
    Weapon,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gi_wishes);
}

#[utoipa::path(
    tag = "gi/wishes/{uid}",
    get,
    path = "/api/gi/wishes/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Wishes", body = Wishes),
    )
)]
#[get("/api/gi/wishes/{uid}")]
async fn get_gi_wishes(
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let is_private = database::gi::connections::get_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);
    let mut forbidden = is_private;

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            let is_admin = database::admins::exists(&username, &pool).await?;
            let has_verified_connection =
                database::gi::connections::get_by_uid_and_username(uid, &username, &pool)
                    .await
                    .is_ok_and(|connection| connection.verified);
            forbidden = gacha_history_forbidden(is_admin, has_verified_connection);
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let language = language_params.lang;

    let beginner = database::gi::wishes::beginner::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Wish::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let standard = database::gi::wishes::standard::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Wish::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let character = database::gi::wishes::character::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Wish::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let weapon = database::gi::wishes::weapon::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Wish::from)
        .collect::<anyhow::Result<Vec<_>>>()?;
    let chronicled = database::gi::wishes::chronicled::get_by_uid(uid, language, &pool)
        .await?
        .into_iter()
        .map(Wish::from)
        .collect::<anyhow::Result<Vec<_>>>()?;

    let wishes = Wishes {
        beginner,
        standard,
        character,
        weapon,
        chronicled,
    };

    Ok(HttpResponse::Ok().json(wishes))
}
