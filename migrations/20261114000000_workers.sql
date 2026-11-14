-- Worker accounts. Each account can log in exactly once: `logged_in_at` is set by
-- that login, and an account that has it set is refused from then on.

CREATE TABLE workers (
    id            TEXT PRIMARY KEY NOT NULL,
    username      TEXT NOT NULL,
    -- Lowercased `username`: "Kenji" and "kenji" are the same account.
    username_key  TEXT NOT NULL UNIQUE,
    -- Argon2id PHC string. Never the password.
    password_hash TEXT NOT NULL,
    created_at    TEXT NOT NULL,
    logged_in_at  TEXT
);
