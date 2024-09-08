SELECT id, user_id, expires_at
FROM session
WHERE id = :id;
