use anyhow::Result;
use sqlx::PgPool;

pub struct DbConnection {
    pub uid: i32,
    pub username: String,
    pub verified: bool,
    pub private: bool,
}

/// Inserts a connection with its supplied privacy value; conflicts update verification only.
pub async fn set(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/connections/set.sql",
        connection.uid,
        connection.username,
        connection.verified,
        connection.private,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Deletes one UID/username connection without deleting its profile or pull history.
pub async fn delete(connection: &DbConnection, pool: &PgPool) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/connections/delete.sql",
        connection.uid,
        connection.username,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Lists every user's connection to this GI UID.
pub async fn get_by_uid(uid: i32, pool: &PgPool) -> Result<Vec<DbConnection>> {
    Ok(
        sqlx::query_file_as!(DbConnection, "sql/gi/connections/get_by_uid.sql", uid,)
            .fetch_all(pool)
            .await?,
    )
}

/// Lists GI connections owned by this username.
pub async fn get_by_username(username: &str, pool: &PgPool) -> Result<Vec<DbConnection>> {
    Ok(sqlx::query_file_as!(
        DbConnection,
        "sql/gi/connections/get_by_username.sql",
        username
    )
    .fetch_all(pool)
    .await?)
}

/// Fetches one connection, returning a database error when it is absent.
pub async fn get_by_uid_and_username(
    uid: i32,
    username: &str,
    pool: &PgPool,
) -> Result<DbConnection> {
    Ok(sqlx::query_file_as!(
        DbConnection,
        "sql/gi/connections/get_by_uid_and_username.sql",
        uid,
        username,
    )
    .fetch_one(pool)
    .await?)
}

/// Updates one connection's privacy flag without performing authorization checks.
pub async fn update_private_by_uid_and_username(
    uid: i32,
    username: &str,
    private: bool,
    pool: &PgPool,
) -> Result<()> {
    sqlx::query_file!(
        "sql/gi/connections/update_private_by_uid_and_username.sql",
        uid,
        username,
        private,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[actix_web::test]
    async fn private_insert_is_preserved_on_verified_update() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let username = format!("c{}", &suffix[..20]);
        let uid = 1_500_000_000 + (uuid::Uuid::new_v4().as_u128() % 100_000_000) as i32;
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, '')")
            .bind(&username)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO gi_profiles (uid, name) VALUES ($1, '')")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
        let mut connection = DbConnection {
            uid,
            username: username.clone(),
            verified: false,
            private: true,
        };
        set(&connection, &pool).await.unwrap();
        assert!(
            get_by_uid_and_username(uid, &username, &pool)
                .await
                .unwrap()
                .private
        );
        connection.private = false;
        connection.verified = true;
        set(&connection, &pool).await.unwrap();
        let row = get_by_uid_and_username(uid, &username, &pool)
            .await
            .unwrap();
        assert!(row.private && row.verified);
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM gi_profiles WHERE uid = $1")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
    }
}
