-- Add the table to store users

CREATE TABLE users
(
    user_id       BIGSERIAL PRIMARY KEY,
    username      TEXT UNIQUE NOT NULL,
    password      TEXT        NOT NULL,
    email_address TEXT -- Is nullable because you might not always want to have an email address associated with a user
);
