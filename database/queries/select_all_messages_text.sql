SELECT message.text
FROM message
LEFT JOIN author
ON message.id = author.message_id
LEFT JOIN user
ON author.user_id = user.id;
