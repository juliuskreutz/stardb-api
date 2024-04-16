ALTER TABLE achievements
    ALTER COLUMN id TYPE integer;

ALTER TABLE achievements_percent
    ALTER COLUMN id TYPE integer;

ALTER TABLE achievements_text
    ALTER COLUMN id TYPE integer;

ALTER TABLE users_achievements_completed
    ALTER COLUMN id TYPE integer;

ALTER TABLE users_achievements_favorites
    ALTER COLUMN id TYPE integer;

ALTER TABLE books
    ALTER COLUMN id TYPE integer;

ALTER TABLE books_percent
    ALTER COLUMN id TYPE integer;

ALTER TABLE books_text
    ALTER COLUMN id TYPE integer;

ALTER TABLE users_books_completed
    ALTER COLUMN id TYPE integer;

ALTER TABLE users_books_favorites
    ALTER COLUMN id TYPE integer;

ALTER TABLE warps
    ALTER COLUMN uid TYPE integer;

ALTER TABLE mihomo
    ALTER COLUMN uid TYPE integer;

ALTER TABLE connections
    ALTER COLUMN uid TYPE integer;

ALTER TABLE scores_achievement
    ALTER COLUMN uid TYPE integer;

ALTER TABLE warps_stats
    ALTER COLUMN uid TYPE integer;

ALTER TABLE warps_stats_4
    ALTER COLUMN uid TYPE integer;

ALTER TABLE warps_stats_5
    ALTER COLUMN uid TYPE integer;

