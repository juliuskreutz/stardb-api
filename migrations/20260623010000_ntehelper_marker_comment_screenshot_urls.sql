ALTER TABLE ntehelper_marker_comment
    ADD COLUMN screenshot_urls jsonb NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE ntehelper_marker_comment
    ADD CONSTRAINT ntehelper_marker_comment_screenshot_urls_check CHECK (
        jsonb_typeof(screenshot_urls) = 'array'
        AND jsonb_array_length(screenshot_urls) <= 4
    );
