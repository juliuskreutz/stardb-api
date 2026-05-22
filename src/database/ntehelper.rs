use anyhow::Result;
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

#[derive(FromRow)]
pub struct DbCompletion {
    pub kind: String,
    pub id: String,
}

#[derive(FromRow)]
pub struct DbSetting {
    pub namespace: String,
    pub data: Value,
}

#[derive(FromRow)]
pub struct DbAchievementStats {
    pub id: String,
    pub completed_users: i64,
    pub total_users: i64,
    pub percent: f64,
}

pub async fn get_completions(username: &str, pool: &PgPool) -> Result<Vec<DbCompletion>> {
    let user_id = get_user_id(username, pool).await?;

    Ok(sqlx::query_as!(
        DbCompletion,
        "SELECT kind, id FROM ntehelper_user_completion WHERE user_id = $1 ORDER BY kind, id",
        user_id,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_settings(username: &str, pool: &PgPool) -> Result<Vec<DbSetting>> {
    let user_id = get_user_id(username, pool).await?;

    Ok(sqlx::query_as!(
        DbSetting,
        r#"SELECT namespace, data as "data: Value" FROM ntehelper_user_settings WHERE user_id = $1 ORDER BY namespace"#,
        user_id,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn replace_state(
    username: &str,
    completions: &[DbCompletion],
    settings: &[DbSetting],
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM ntehelper_user_completion WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "DELETE FROM ntehelper_user_settings WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    for completion in completions {
        insert_completion(user_id, &completion.kind, &completion.id, &mut tx).await?;
    }

    for setting in settings {
        upsert_setting(user_id, &setting.namespace, &setting.data, &mut tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn patch_completions(
    username: &str,
    add: &[DbCompletion],
    remove: &[DbCompletion],
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;
    let mut tx = pool.begin().await?;

    for completion in add {
        insert_completion(user_id, &completion.kind, &completion.id, &mut tx).await?;
    }

    for completion in remove {
        sqlx::query!(
            "DELETE FROM ntehelper_user_completion WHERE user_id = $1 AND kind = $2 AND id = $3",
            user_id,
            &completion.kind,
            &completion.id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn set_setting(
    username: &str,
    namespace: &str,
    data: &Value,
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;

    sqlx::query!(
        "INSERT INTO ntehelper_user_settings (user_id, namespace, data) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, namespace) DO UPDATE SET data = EXCLUDED.data, updated_at = now()",
        user_id,
        namespace,
        data,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn achievement_stats(pool: &PgPool) -> Result<Vec<DbAchievementStats>> {
    Ok(sqlx::query_as!(
        DbAchievementStats,
        r#"WITH total AS (
             SELECT COUNT(DISTINCT user_id)::bigint AS total_users
             FROM ntehelper_user_completion
             WHERE kind = 'achievement'
         )
         SELECT
             id AS "id!",
             COUNT(DISTINCT user_id)::bigint AS "completed_users!",
             total.total_users AS "total_users!",
             CASE
                 WHEN total.total_users = 0 THEN 0::float8
                 ELSE COUNT(DISTINCT user_id)::float8 / total.total_users::float8
             END AS "percent!"
         FROM ntehelper_user_completion
         CROSS JOIN total
         WHERE kind = 'achievement'
         GROUP BY id, total.total_users
         ORDER BY id"#,
    )
    .fetch_all(pool)
    .await?)
}

async fn insert_completion(
    user_id: i64,
    kind: &str,
    id: &str,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO ntehelper_user_completion (user_id, kind, id) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, kind, id) DO NOTHING",
        user_id,
        kind,
        id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn upsert_setting(
    user_id: i64,
    namespace: &str,
    data: &Value,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO ntehelper_user_settings (user_id, namespace, data) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, namespace) DO UPDATE SET data = EXCLUDED.data, updated_at = now()",
        user_id,
        namespace,
        data,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn get_user_id(username: &str, pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}
