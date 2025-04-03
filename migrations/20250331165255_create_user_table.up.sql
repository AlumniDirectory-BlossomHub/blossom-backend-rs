CREATE TABLE "user"
(
    id          SERIAL PRIMARY KEY,
    email       VARCHAR(255) UNIQUE NOT NULL,
    password    VARCHAR(255),
    admin_level SMALLINT                 DEFAULT 0,
    username    VARCHAR(255)        NOT NULL,
    avatar_id   VARCHAR(36)              DEFAULT NULL,
    status      SMALLINT                 DEFAULT 0,
    created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

--每个库创建一次即可
CREATE OR REPLACE FUNCTION upd_timestamp() RETURNS TRIGGER AS
$$
BEGIN
    new.updated_at = CURRENT_TIMESTAMP;
    RETURN new;
END
$$
    LANGUAGE plpgsql;

CREATE TRIGGER user_upd_trigger
    BEFORE UPDATE
    ON "user"
    FOR EACH ROW
EXECUTE PROCEDURE upd_timestamp();
