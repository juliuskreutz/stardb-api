ALTER TABLE ONLY users_achievements RENAME TO users_achievements_completed;

ALTER TABLE ONLY users_achievements_completed RENAME CONSTRAINT users_achievements_pkey TO users_achievements_completed_pkey;

ALTER TABLE ONLY users_achievements_completed RENAME CONSTRAINT users_achievements_id_fkey TO users_achievements_completed_id_fkey;

ALTER TABLE ONLY users_achievements_completed RENAME CONSTRAINT users_achievements_username_fkey TO users_achievements_completed_username_fkey;

CREATE TABLE IF NOT EXISTS users_achievements_favorites (
    username text NOT NULL,
    id bigint NOT NULL
);

ALTER TABLE ONLY users_achievements_favorites
    ADD CONSTRAINT users_achievements_favorites_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY users_achievements_favorites
    ADD CONSTRAINT users_achievements_favorites_id_fkey FOREIGN KEY (id) REFERENCES achievements (id) ON DELETE CASCADE;

ALTER TABLE ONLY users_achievements_favorites
    ADD CONSTRAINT users_achievements_favorites_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY users_books RENAME TO users_books_completed;

ALTER TABLE ONLY users_books_completed RENAME CONSTRAINT users_books_pkey TO users_books_completed_pkey;

ALTER TABLE ONLY users_books_completed RENAME CONSTRAINT users_books_id_fkey TO users_books_completed_id_fkey;

ALTER TABLE ONLY users_books_completed RENAME CONSTRAINT users_books_username_fkey TO users_books_completed_username_fkey;

CREATE TABLE IF NOT EXISTS users_books_favorites (
    username text NOT NULL,
    id bigint NOT NULL
);

ALTER TABLE ONLY users_books_favorites
    ADD CONSTRAINT users_books_favorites_pkey PRIMARY KEY (username, id);

ALTER TABLE ONLY users_books_favorites
    ADD CONSTRAINT users_books_favorites_id_fkey FOREIGN KEY (id) REFERENCES books (id) ON DELETE CASCADE;

ALTER TABLE ONLY users_books_favorites
    ADD CONSTRAINT users_books_favorites_username_fkey FOREIGN KEY (username) REFERENCES users (username) ON UPDATE CASCADE ON DELETE CASCADE;

