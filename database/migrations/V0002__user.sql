CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE author (
    id SERIAL PRIMARY KEY,
    user_id INT,
    message_id INT,
    FOREIGN KEY(user_id) REFERENCES user(id),
    FOREIGN KEY(message_id) REFERENCES message(id)
);
