-- The official wish API reports Europe as `os_euro`. The importer previously
-- missed that value and interpreted its local timestamps as UTC+8 instead of
-- UTC+1, storing the resulting instants seven hours too early.

UPDATE gi_wishes_beginner
SET timestamp = timestamp + INTERVAL '7 hours'
WHERE uid >= 700000000 AND uid < 800000000 AND official;

UPDATE gi_wishes_standard
SET timestamp = timestamp + INTERVAL '7 hours'
WHERE uid >= 700000000 AND uid < 800000000 AND official;

UPDATE gi_wishes_character
SET timestamp = timestamp + INTERVAL '7 hours'
WHERE uid >= 700000000 AND uid < 800000000 AND official;

UPDATE gi_wishes_weapon
SET timestamp = timestamp + INTERVAL '7 hours'
WHERE uid >= 700000000 AND uid < 800000000 AND official;

UPDATE gi_wishes_chronicled
SET timestamp = timestamp + INTERVAL '7 hours'
WHERE uid >= 700000000 AND uid < 800000000 AND official;
