SELECT id, user_id, expires_at
FROM session
ORDER BY id DESC
LIMIT 1;
