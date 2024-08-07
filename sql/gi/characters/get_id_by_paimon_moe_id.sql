SELECT
    id
FROM
    gi_characters_text
WHERE
    lower(replace(replace(name, '''', ''), ' ', '_')) = $1;

