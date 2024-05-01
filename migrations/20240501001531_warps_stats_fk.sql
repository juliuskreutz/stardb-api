ALTER TABLE ONLY warps_stats
    ADD CONSTRAINT warps_stats_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_stats_4
    ADD CONSTRAINT warps_stats_4_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

ALTER TABLE ONLY warps_stats_5
    ADD CONSTRAINT warps_stats_5_uid_fkey FOREIGN KEY (uid) REFERENCES mihomo (uid) ON DELETE CASCADE;

