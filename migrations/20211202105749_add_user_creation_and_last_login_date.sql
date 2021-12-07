-- Add migration script here

ALTER TABLE users
    ADD creation_time BIGINT NOT NULL;

ALTER TABLE users
    ADD last_login_time BIGINT NOT NULL;
