-- Add up migration script here
CREATE TABLE IF NOT EXISTS "user_table" (
    id uuid DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS "admin_table" (
    id uuid DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS "course_table" (
    id uuid DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    image_url VARCHAR(255),
    price INTEGER CHECK (price >= 0),
    admin_id uuid NOT NULL,
    PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS "purchases_table" (
    id uuid DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL,
    admin_id uuid NOT NULL,
    PRIMARY KEY(id)
);