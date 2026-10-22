-- HSR Fate collab banner pools use the dedicated collab gacha types.
UPDATE banners
SET character_gacha_type = 21
WHERE character IN (1014, 1015, 1508, 1509);

UPDATE banners
SET light_cone_gacha_type = 22
WHERE light_cone IN (23045, 23046, 23061, 23062);
