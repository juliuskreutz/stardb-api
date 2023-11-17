CREATE TABLE IF NOT EXISTS temporary_challenge_leaderboard (
    username TEXT PRIMARY KEY NOT NULL REFERENCES users ON DELETE CASCADE,
    timestamp TIMESTAMP NOT NULL
);