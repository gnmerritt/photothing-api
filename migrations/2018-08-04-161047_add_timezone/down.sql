-- users
ALTER TABLE users ALTER COLUMN updated_at TYPE TIMESTAMP;
ALTER TABLE users ALTER COLUMN joined TYPE TIMESTAMP;
-- photos
ALTER TABLE photos ALTER COLUMN created_at TYPE TIMESTAMP;
ALTER TABLE photos ALTER COLUMN updated_at TYPE TIMESTAMP;