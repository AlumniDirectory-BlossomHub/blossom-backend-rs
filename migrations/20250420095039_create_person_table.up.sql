CREATE TYPE gender AS ENUM ('male', 'female');

CREATE TABLE person
(
    id         SERIAL PRIMARY KEY,
    name       VARCHAR(32),
    birthday   DATE                     DEFAULT NULL,
    gender     gender NOT NULL,
    photo_id   VARCHAR(36)              DEFAULT NULL,
    phone      VARCHAR(24)              DEFAULT NULL,
    email      VARCHAR(255)             DEFAULT NULL,
    qq         VARCHAR(16)              DEFAULT NULL,
    wechat     VARCHAR(64)              DEFAULT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER person_upd_trigger
    BEFORE UPDATE
    ON person
    FOR EACH ROW
EXECUTE PROCEDURE upd_timestamp();