CREATE TABLE ntehelper_marker_comment_vote (
    comment_id bigint NOT NULL,
    user_id bigint NOT NULL,
    value integer NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (comment_id, user_id),
    CONSTRAINT ntehelper_marker_comment_vote_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES ntehelper_marker_comment (id) ON DELETE CASCADE,
    CONSTRAINT ntehelper_marker_comment_vote_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT ntehelper_marker_comment_vote_value_check CHECK (value IN (-1, 1))
);

CREATE INDEX ntehelper_marker_comment_vote_user_idx
    ON ntehelper_marker_comment_vote (user_id, updated_at DESC);

CREATE INDEX ntehelper_marker_comment_user_created_idx
    ON ntehelper_marker_comment (user_id, created_at DESC);
