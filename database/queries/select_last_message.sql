SELECT message.id, message.text, user.name
FROM message
LEFT JOIN user
ON message.created_by_id = user.id
ORDER BY message.id DESC LIMIT 1;
