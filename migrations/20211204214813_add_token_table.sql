-- Add migration script here
CREATE TABLE tokens
(
    user_id BIGINT,
    token   TEXT,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
)