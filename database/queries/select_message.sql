SELECT message.id, message.text, user.id, user.name
FROM message
LEFT JOIN user
ON message.created_by_id = user.id
WHERE message.id = :message_id
