use sqlx::PgPool;

use crate::Result;

#[derive(Default)]
pub struct DbSeries {
    pub id: i32,
    pub name: String,
    pub priority: i32,
}

pub async fn set_series(series: &DbSeries, pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "
        INSERT INTO
            series(id, name, priority)
        VALUES
            ($1, $2, $3)
        ON CONFLICT
            (id)
        DO UPDATE SET
            name = EXCLUDED.name,
            priority = EXCLUDED.priority
        ",
        series.id,
        series.name,
        series.priority,
    )
    .execute(pool)
    .await?;

    Ok(())
}
