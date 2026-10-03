CREATE TABLE users (
    email TEXT UNIQUE PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
);