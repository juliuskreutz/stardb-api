DELETE FROM connections a USING (
    SELECT
        min(ctid) AS ctid,
        uid,
        username
    FROM
        connections
    GROUP BY
        (uid,
            username)
    HAVING
        count(*) > 1) b
WHERE
    a.uid = b.uid
    AND a.username = b.username
    AND a.ctid <> b.ctid;

ALTER TABLE ONLY connections
    ADD CONSTRAINT connections_pkey PRIMARY KEY (uid, username);

