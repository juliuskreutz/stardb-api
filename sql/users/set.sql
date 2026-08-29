INSERT INTO users (username, PASSWORD, email)
    VALUES ($1, $2, LOWER($3));

