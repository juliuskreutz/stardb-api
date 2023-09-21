ALTER TABLE admins DROP CONSTRAINT admins_username_fkey;
ALTER TABLE admins ADD CONSTRAINT admins_username_fkey FOREIGN KEY (username) REFERENCES users ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE sessions DROP CONSTRAINT sessions_new_username_fkey;
ALTER TABLE sessions ADD CONSTRAINT sessions_username_fkey FOREIGN KEY (username) REFERENCES users ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE users_achievements DROP CONSTRAINT completed_username_fkey;
ALTER TABLE users_achievements ADD CONSTRAINT users_achievements_username_fkey FOREIGN KEY (username) REFERENCES users ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE users_books DROP CONSTRAINT users_books_username_fkey;
ALTER TABLE users_books ADD CONSTRAINT users_books_username_fkey FOREIGN KEY (username) REFERENCES users ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE verifications DROP CONSTRAINT verifications_username_fkey;
ALTER TABLE verifications ADD CONSTRAINT verifications_username_fkey FOREIGN KEY (username) REFERENCES users ON DELETE CASCADE ON UPDATE CASCADE;
