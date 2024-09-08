CREATE TABLE session (
    id TEXT PRIMARY KEY, -- UUID
    user_id INT, -- The session's user, can be null
    expires_at BIGINT, -- Timestamp
    FOREIGN KEY(user_id) REFERENCES user(id)
);
