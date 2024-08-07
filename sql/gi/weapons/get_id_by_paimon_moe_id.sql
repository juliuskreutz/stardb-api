SELECT
    id
FROM
    gi_weapons_text
WHERE
    lower(replace(replace(name, '''', ''), ' ', '_')) = $1;

