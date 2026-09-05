//! Atomic achievement-list mutations. Identifiers come exclusively from these enums.
use anyhow::{bail, Result};
use sqlx::{PgConnection, PgPool};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub enum Game {
    Hsr,
    Gi,
    Zzz,
}
#[derive(Clone, Copy)]
pub enum List {
    Completed,
    Favorites,
}
impl Game {
    fn prefix(self) -> &'static str {
        match self {
            Self::Hsr => "",
            Self::Gi => "gi_",
            Self::Zzz => "zzz_",
        }
    }
}
impl List {
    fn suffix(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Favorites => "favorites",
        }
    }
}

/// Build relationships from the full catalog before applying visibility filters.
pub fn related_by_set(
    rows: impl IntoIterator<Item = (i32, Option<i32>)>,
) -> HashMap<i32, Vec<i32>> {
    let mut related = HashMap::<i32, Vec<i32>>::new();
    for (id, set) in rows {
        if let Some(set) = set {
            related.entry(set).or_default().push(id);
        }
    }
    related
}

fn select_ids(ids: &[i32], rows: &[(i32, bool, Option<i32>)], completed: bool) -> Result<Vec<i32>> {
    let catalog: HashMap<_, _> = rows
        .iter()
        .map(|&(id, impossible, set)| (id, (impossible, set)))
        .collect();
    let mut sets = HashSet::new();
    let mut seen = HashSet::new();
    let mut selected = Vec::new();
    for &id in ids.iter().rev() {
        let Some(&(impossible, set)) = catalog.get(&id) else {
            bail!("Unknown achievement id {id}");
        };
        if completed && impossible {
            continue;
        }
        if !seen.insert(id) {
            continue;
        }
        if let Some(set) = set {
            if !sets.insert(set) {
                continue;
            }
        }
        selected.push(id);
    }
    selected.reverse();
    Ok(selected)
}

pub async fn apply(
    game: Game,
    list: List,
    username: &str,
    ids: &[i32],
    replace: bool,
    conn: &mut PgConnection,
) -> Result<()> {
    // Serialize competing updates for this user, including full imports.
    sqlx::query("SELECT username FROM users WHERE username = $1 FOR UPDATE")
        .bind(username)
        .fetch_optional(&mut *conn)
        .await?;
    let catalog = format!("{}achievements", game.prefix());
    let table = format!("{}users_achievements_{}", game.prefix(), list.suffix());
    let rows = sqlx::query_as::<_, (i32, bool, Option<i32>)>(&format!(
        "SELECT id, impossible, \"set\" FROM {catalog} WHERE id = ANY($1)"
    ))
    .bind(ids)
    .fetch_all(&mut *conn)
    .await?;
    let selected = select_ids(ids, &rows, matches!(list, List::Completed))?;
    if replace {
        sqlx::query(&format!("DELETE FROM {table} WHERE username = $1"))
            .bind(username)
            .execute(&mut *conn)
            .await?;
    }
    sqlx::query(&format!("DELETE FROM {table} u USING {catalog} a WHERE u.username = $1 AND u.id = a.id AND a.\"set\" IN (SELECT \"set\" FROM {catalog} WHERE id = ANY($2)) AND NOT (u.id = ANY($2))"))
        .bind(username).bind(&selected).execute(&mut *conn).await?;
    sqlx::query(&format!("INSERT INTO {table}(username, id) SELECT $1, UNNEST($2::int[]) ON CONFLICT(username, id) DO NOTHING"))
        .bind(username).bind(&selected).execute(&mut *conn).await?;
    Ok(())
}

pub async fn add_all(
    game: Game,
    list: List,
    username: &str,
    ids: &[i32],
    pool: &PgPool,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    apply(game, list, username, ids, false, &mut tx).await?;
    tx.commit().await?;
    Ok(())
}
pub async fn delete_all(
    game: Game,
    list: List,
    username: &str,
    ids: &[i32],
    pool: &PgPool,
) -> Result<()> {
    let table = format!("{}users_achievements_{}", game.prefix(), list.suffix());
    sqlx::query(&format!(
        "DELETE FROM {table} WHERE username = $1 AND id = ANY($2)"
    ))
    .bind(username)
    .bind(ids)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relations_include_members_before_visibility_filtering() {
        let full_catalog = [(1, Some(7), false), (2, Some(7), true), (3, None, false)];
        let related = related_by_set(
            full_catalog
                .iter()
                .map(|&(id, set, _hidden_impossible)| (id, set)),
        );
        let visible: Vec<_> = full_catalog.iter().filter(|a| !a.2).collect();
        assert_eq!(visible.len(), 2);
        assert_eq!(
            related[&7]
                .iter()
                .copied()
                .filter(|id| *id != visible[0].0)
                .collect::<Vec<_>>(),
            vec![2]
        );
    }
    #[test]
    fn last_possible_member_wins_and_unknown_ids_fail() {
        let rows = [
            (1, false, Some(7)),
            (2, false, None),
            (3, false, Some(7)),
            (4, true, Some(7)),
        ];
        assert_eq!(select_ids(&[1, 2, 3, 4], &rows, true).unwrap(), vec![2, 3]);
        assert_eq!(select_ids(&[1, 2, 3, 4], &rows, false).unwrap(), vec![2, 4]);
        assert_eq!(select_ids(&[3, 2, 1, 1], &rows, true).unwrap(), vec![2, 1]);
        assert!(select_ids(&[1, 99], &rows, true).is_err());
    }
}

#[cfg(test)]
mod database_tests {
    use super::*;
    use sqlx::Connection;

    async fn connection() -> PgConnection {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        let mut conn = PgConnection::connect(&url).await.unwrap();
        // Session-local tables exercise actual production mutation SQL without fixture collisions.
        sqlx::query("CREATE TEMP TABLE users(username text PRIMARY KEY)")
            .execute(&mut conn)
            .await
            .unwrap();
        sqlx::query("INSERT INTO users VALUES ('achievement_test')")
            .execute(&mut conn)
            .await
            .unwrap();
        for game in [Game::Hsr, Game::Gi, Game::Zzz] {
            let catalog = format!("{}achievements", game.prefix());
            sqlx::query(&format!("CREATE TEMP TABLE {catalog}(id int PRIMARY KEY, impossible bool NOT NULL, \"set\" int)")).execute(&mut conn).await.unwrap();
            sqlx::query(&format!(
                "INSERT INTO {catalog} VALUES (1,false,7),(2,false,NULL),(3,false,7),(4,true,7)"
            ))
            .execute(&mut conn)
            .await
            .unwrap();
            for list in [List::Completed, List::Favorites] {
                let table = format!("{}users_achievements_{}", game.prefix(), list.suffix());
                sqlx::query(&format!("CREATE TEMP TABLE {table}(username text REFERENCES users, id int REFERENCES {catalog}, PRIMARY KEY(username,id))")).execute(&mut conn).await.unwrap();
            }
        }
        conn
    }
    async fn ids(game: Game, list: List, conn: &mut PgConnection) -> Vec<i32> {
        sqlx::query_scalar(&format!(
            "SELECT id FROM {}users_achievements_{} ORDER BY id",
            game.prefix(),
            list.suffix()
        ))
        .fetch_all(conn)
        .await
        .unwrap()
    }
    #[actix_web::test]
    async fn all_games_batch_eviction_impossible_and_unknown_are_atomic() {
        let mut conn = connection().await;
        for game in [Game::Hsr, Game::Gi, Game::Zzz] {
            for list in [List::Completed, List::Favorites] {
                let mut tx = conn.begin().await.unwrap();
                apply(game, list, "achievement_test", &[1], false, &mut tx)
                    .await
                    .unwrap();
                tx.commit().await.unwrap();
                let mut tx = conn.begin().await.unwrap();
                apply(game, list, "achievement_test", &[2, 3, 4], false, &mut tx)
                    .await
                    .unwrap();
                tx.commit().await.unwrap();
                let expected = if matches!(list, List::Completed) {
                    vec![2, 3]
                } else {
                    vec![2, 4]
                };
                assert_eq!(ids(game, list, &mut conn).await, expected);
                let mut tx = conn.begin().await.unwrap();
                assert!(
                    apply(game, list, "achievement_test", &[1, 99], true, &mut tx)
                        .await
                        .is_err()
                );
                tx.rollback().await.unwrap();
                assert_eq!(ids(game, list, &mut conn).await, expected);
            }
        }
    }
    #[actix_web::test]
    async fn failed_cross_game_full_import_preserves_original_lists() {
        let mut conn = connection().await;
        let mut tx = conn.begin().await.unwrap();
        for game in [Game::Hsr, Game::Gi] {
            apply(
                game,
                List::Completed,
                "achievement_test",
                &[1],
                false,
                &mut tx,
            )
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();
        let mut tx = conn.begin().await.unwrap();
        apply(
            Game::Hsr,
            List::Completed,
            "achievement_test",
            &[2],
            true,
            &mut tx,
        )
        .await
        .unwrap();
        assert!(apply(
            Game::Gi,
            List::Completed,
            "achievement_test",
            &[99],
            true,
            &mut tx
        )
        .await
        .is_err());
        tx.rollback().await.unwrap();
        for game in [Game::Hsr, Game::Gi] {
            assert_eq!(ids(game, List::Completed, &mut conn).await, vec![1]);
        }
    }
    #[actix_web::test]
    async fn omitted_put_columns_preserve_stored_metadata_in_all_games() {
        let mut conn = connection().await;
        for (game, sql) in [
            (
                Game::Hsr,
                include_str!("../../sql/achievements/update_achievement_by_id.sql"),
            ),
            (
                Game::Gi,
                include_str!("../../sql/gi/achievements/update_achievement_by_id.sql"),
            ),
            (
                Game::Zzz,
                include_str!("../../sql/zzz/achievements/update_achievement_by_id.sql"),
            ),
        ] {
            let table = format!("{}achievements", game.prefix());
            sqlx::query(&format!("ALTER TABLE {table} ADD version text DEFAULT '1.0', ADD comment text DEFAULT 'old', ADD reference text DEFAULT 'ref', ADD difficulty text DEFAULT 'easy', ADD video text DEFAULT 'video', ADD gacha bool DEFAULT true, ADD timegated text DEFAULT 'time', ADD missable bool DEFAULT true")).execute(&mut conn).await.unwrap();
            sqlx::query(sql)
                .bind(1i32)
                .bind(None::<String>)
                .bind(Some("new"))
                .bind(None::<String>)
                .bind(None::<String>)
                .bind(None::<String>)
                .bind(None::<bool>)
                .bind(None::<String>)
                .bind(None::<bool>)
                .bind(None::<bool>)
                .bind(None::<i32>)
                .execute(&mut conn)
                .await
                .unwrap();
            let row: (String, String, bool, i32) = sqlx::query_as(&format!(
                "SELECT version, comment, gacha, \"set\" FROM {table} WHERE id = 1"
            ))
            .fetch_one(&mut conn)
            .await
            .unwrap();
            assert_eq!(row, ("1.0".into(), "new".into(), true, 7));
        }
    }
}
