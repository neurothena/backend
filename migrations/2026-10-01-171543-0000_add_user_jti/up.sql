CREATE TABLE user_jti (
    jti UUID PRIMARY KEY,
    user_email TEXT NOT NULL,

    FOREIGN KEY (user_email) REFERENCES users(email) ON DELETE CASCADE ON UPDATE CASCADE
);
