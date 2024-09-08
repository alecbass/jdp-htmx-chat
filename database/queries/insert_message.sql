INSERT INTO message (text) VALUES (:message);

INSERT INTO author (user_id, message_id) VALUES (last_insert_rowid(), :userId);
