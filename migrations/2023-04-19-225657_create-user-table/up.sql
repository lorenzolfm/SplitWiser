CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    lightning_address TEXT,
    created_at TIMESTAMPTZ NOT NULL
);
