ALTER TABLE warps
DROP CONSTRAINT warps_pkey;

ALTER TABLE warps
ADD PRIMARY KEY (id, timestamp);
