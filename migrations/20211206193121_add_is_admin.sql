-- Add migration script here
ALTER TABLE users
    ADD is_admin BOOLEAN NOT NULL DEFAULT FALSE