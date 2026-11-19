-- Before April 2025 the rng.moe importer passed millisecond timestamps to a
-- seconds-based conversion, storing pulls in the years 56475-57223. Divide
-- those instants back down; every affected value is a whole number of
-- milliseconds and maps into July 2024 - April 2025. Genuine pulls never
-- reach year 3000, so the cutoff cannot touch correct rows.

UPDATE zzz_signals_standard
SET timestamp = to_timestamp(extract(epoch FROM timestamp) / 1000)
WHERE timestamp >= '3000-01-01';

UPDATE zzz_signals_special
SET timestamp = to_timestamp(extract(epoch FROM timestamp) / 1000)
WHERE timestamp >= '3000-01-01';

UPDATE zzz_signals_w_engine
SET timestamp = to_timestamp(extract(epoch FROM timestamp) / 1000)
WHERE timestamp >= '3000-01-01';

UPDATE zzz_signals_bangboo
SET timestamp = to_timestamp(extract(epoch FROM timestamp) / 1000)
WHERE timestamp >= '3000-01-01';

UPDATE zzz_signals_exclusive_rescreening
SET timestamp = to_timestamp(extract(epoch FROM timestamp) / 1000)
WHERE timestamp >= '3000-01-01';

UPDATE zzz_signals_w_engine_reverberation
SET timestamp = to_timestamp(extract(epoch FROM timestamp) / 1000)
WHERE timestamp >= '3000-01-01';
