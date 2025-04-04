CREATE TABLE "user_verification_token"
(
    user_id INTEGER REFERENCES "user" (id),
    token   UUID UNIQUE,
    expire  TIMESTAMP WITH TIME ZONE
);