INSERT INTO zzz_banners (id, name, start, "end", character, w_engine, bangboo)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (id)
    DO UPDATE SET
        name = excluded.name,
        start = excluded.start,
        "end" = excluded."end",
        character = excluded.character,
        w_engine = excluded.w_engine,
        bangboo = excluded.bangboo;

