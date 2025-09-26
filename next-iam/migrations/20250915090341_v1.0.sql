-- Add migration script here
INSERT INTO users (id, name, email, created_at)
VALUES 
    (gen_random_uuid(), 'Alice', 'alice@example.com', now()),
    (gen_random_uuid(), 'Bob',   'bob@example.com',   now());