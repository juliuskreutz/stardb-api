use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres};

#[derive(Clone)]
pub struct DbBanner {
    pub id: i32,
    pub name: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub character: Option<i32>,
    pub character_gacha_type: Option<i32>,
    pub w_engine: Option<i32>,
    pub w_engine_gacha_type: Option<i32>,
    pub bangboo: Option<i32>,
    pub bangboo_gacha_type: Option<i32>,
}

pub async fn set<'e, E>(banner: &DbBanner, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!(
        "sql/zzz/banners/set.sql",
        banner.id,
        banner.name,
        banner.start,
        banner.end,
        banner.character,
        banner.character_gacha_type,
        banner.w_engine,
        banner.w_engine_gacha_type,
        banner.bangboo,
        banner.bangboo_gacha_type,
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<DbBanner>> {
    get_all_with_executor(pool).await
}

pub async fn get_all_with_executor<'e, E>(executor: E) -> Result<Vec<DbBanner>>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/zzz/banners/get_all.sql")
            .fetch_all(executor)
            .await?,
    )
}

pub async fn get_by_id(id: i32, pool: &PgPool) -> Result<DbBanner> {
    get_by_id_with_executor(id, pool).await
}

pub async fn get_by_id_with_executor<'e, E>(id: i32, executor: E) -> Result<DbBanner>
where
    E: Executor<'e, Database = Postgres>,
{
    Ok(
        sqlx::query_file_as!(DbBanner, "sql/zzz/banners/get_by_id.sql", id)
            .fetch_one(executor)
            .await?,
    )
}

pub async fn delete_by_id<'e, E>(id: i32, executor: E) -> Result<()>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_file!("sql/zzz/banners/delete_by_id.sql", id)
        .execute(executor)
        .await?;
    Ok(())
}

#[cfg(test)]
mod zzz_banner {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgConnection;
    use uuid::Uuid;

    async fn assert_rejected(banner: &DbBanner, connection: &mut PgConnection) {
        sqlx::query("SAVEPOINT invalid_banner")
            .execute(&mut *connection)
            .await
            .unwrap();
        assert!(set(banner, &mut *connection).await.is_err());
        sqlx::query("ROLLBACK TO SAVEPOINT invalid_banner")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::query("RELEASE SAVEPOINT invalid_banner")
            .execute(connection)
            .await
            .unwrap();
    }

    #[actix_web::test]
    async fn crud_and_validation() {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("test database connects");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("test database is migrated");

        let suffix = (Uuid::new_v4().as_u128() % 100_000_000) as i32;
        let item = 1_700_000_000 + suffix;
        let banner_id = 1_800_000_000 + suffix;
        let start = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
        let end = start + Duration::days(1);
        let mut transaction = pool.begin().await.unwrap();

        sqlx::query("INSERT INTO zzz_characters (id, rarity) VALUES ($1, 4)")
            .bind(item)
            .execute(&mut *transaction)
            .await
            .unwrap();
        let banner = DbBanner {
            id: banner_id,
            name: "test".to_string(),
            start,
            end,
            character: Some(item),
            character_gacha_type: Some(2),
            w_engine: None,
            w_engine_gacha_type: None,
            bangboo: None,
            bangboo_gacha_type: None,
        };

        assert_rejected(
            &DbBanner {
                end: start,
                ..banner.clone()
            },
            &mut transaction,
        )
        .await;
        assert_rejected(
            &DbBanner {
                id: banner_id + 1,
                character: None,
                character_gacha_type: None,
                ..banner.clone()
            },
            &mut transaction,
        )
        .await;

        set(&banner, &mut *transaction).await.unwrap();
        let stored = get_by_id_with_executor(banner_id, &mut *transaction)
            .await
            .unwrap();
        assert_eq!(stored.character, Some(item));
        assert_eq!(stored.character_gacha_type, Some(2));

        delete_by_id(banner_id, &mut *transaction).await.unwrap();
        let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM zzz_banners WHERE id = $1")
            .bind(banner_id)
            .fetch_one(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(remaining, 0);

        transaction.rollback().await.unwrap();
    }
}
