use std::env;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "free-jade-alert")),
    paths(post_free_jade_alert),
    components(schemas(
        FreeJadeAlert,
        Code,
        Reward,
        RewardType,
        Link
    ))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct FreeJadeAlert {
    title: String,
    description: String,
    duration: String,
    #[serde(default)]
    codes: Vec<Code>,
    rewards: Vec<Reward>,
    links: Vec<Link>,
    image: Option<String>,
}

#[derive(Deserialize, ToSchema)]
struct Code {
    code: String,
    rewards: Vec<Reward>,
}

#[derive(Deserialize, ToSchema)]
struct Reward {
    r#type: RewardType,
    amount: usize,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum RewardType {
    Jade,
    Credit,
    Exp,
    Custom(String),
}

#[derive(Deserialize, ToSchema)]
struct Link {
    name: String,
    link: String,
}

#[derive(Serialize)]
struct Webhook {
    embeds: Vec<Embed>,
}

#[derive(Serialize)]
struct Embed {
    title: String,
    url: String,
    color: i32,
    description: String,
    fields: Vec<Field>,
    image: Option<Image>,
    footer: Footer,
    timestamp: NaiveDateTime,
}

#[derive(Serialize)]
struct Field {
    name: String,
    value: String,
}
#[derive(Serialize)]
struct Image {
    url: String,
}

#[derive(Serialize)]
struct Footer {
    text: String,
}

impl From<FreeJadeAlert> for Webhook {
    fn from(free_jade_alert: FreeJadeAlert) -> Self {
        let title = free_jade_alert.title;
        let url = "https://stardb.gg/free-jade-alert".to_string();
        let color = 5814783;
        let description = free_jade_alert.description;

        let mut fields = Vec::new();

        fields.push(Field {
            name: "Duration".to_string(),
            value: free_jade_alert.duration,
        });

        for code in free_jade_alert.codes {
            fields.push(Field {
                name: "Code".to_string(),
                value: format!("```{}```", code.code),
            });

            let mut rewards = String::new();

            for reward in code.rewards {
                rewards.push_str(&format!("- {} ", reward.amount));

                match reward.r#type {
                    RewardType::Jade => rewards.push_str("<:StellarJade:1109548953931886708>"),
                    RewardType::Credit => rewards.push_str("<:Credits:1114861682985013288>"),
                    RewardType::Exp => rewards.push_str("<:XpBook:1114861653301932062>"),
                    RewardType::Custom(s) => rewards.push_str(&s),
                };

                rewards.push('\n');
            }

            fields.push(Field {
                name: "Rewards".to_string(),
                value: rewards,
            });
        }

        let mut rewards = String::new();

        for reward in free_jade_alert.rewards {
            rewards.push_str(&format!("- {} ", reward.amount));

            match reward.r#type {
                RewardType::Jade => rewards.push_str("<:StellarJade:1109548953931886708>"),
                RewardType::Credit => rewards.push_str("<:Credits:1114861682985013288>"),
                RewardType::Exp => rewards.push_str("<:XpBook:1114861653301932062>"),
                RewardType::Custom(s) => rewards.push_str(&s),
            };

            rewards.push('\n');
        }

        fields.push(Field {
            name: "Rewards".to_string(),
            value: rewards,
        });

        let mut links = String::new();

        for link in free_jade_alert.links {
            links.push_str(&format!(
                "- [{}]({}?utm_source=stardb.gg)\n",
                link.name, link.link
            ));
        }

        fields.push(Field {
            name: "Links".to_string(),
            value: links,
        });

        let image = free_jade_alert.image.map(|s| Image { url: s });

        let footer = Footer {
            text: "Published".to_string(),
        };

        let timestamp = Utc::now().naive_utc();

        Self {
            embeds: vec![Embed {
                title,
                url,
                color,
                description,
                fields,
                image,
                footer,
                timestamp,
            }],
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_free_jade_alert);
}

#[utoipa::path(
    tag = "free-jade-alert",
    post,
    path = "/api/free-jade-alert",
    responses(
        (status = 200, description = "Success"),
    )
)]
#[post("/api/free-jade-alert")]
async fn post_free_jade_alert(
    session: Session,
    free_jade_alert: web::Json<FreeJadeAlert>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let client = reqwest::Client::new();

    let webhook: Webhook = free_jade_alert.0.into();

    client
        .post(env::var("DISCORD_WEBHOOK")?)
        .json(&webhook)
        .send()
        .await?;

    Ok(HttpResponse::Ok().finish())
}
