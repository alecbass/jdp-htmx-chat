CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL
);

ALTER TABLE message
ADD COLUMN created_by_id INT NOT NULL REFERENCES user(id);
