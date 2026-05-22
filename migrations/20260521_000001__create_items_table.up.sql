CREATE TABLE items (
    id         uuid        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    content    text        NOT NULL,
    user_id    uuid        NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    created_at timestamptz NOT NULL DEFAULT NOW()
);

CREATE INDEX items_created_at_idx ON items (created_at DESC);
CREATE INDEX items_user_id_idx    ON items (user_id);
