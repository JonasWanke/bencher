PRAGMA foreign_keys = off;
CREATE TABLE up_user (
    id INTEGER PRIMARY KEY NOT NULL,
    uuid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    admin BOOLEAN NOT NULL,
    locked BOOLEAN NOT NULL,
    created BIGINT NOT NULL,
    modified BIGINT NOT NULL
);
INSERT INTO up_user(
        id,
        uuid,
        name,
        slug,
        email,
        admin,
        locked,
        created,
        modified
    )
SELECT id,
    uuid,
    name,
    slug,
    email,
    admin,
    locked,
    (
        SELECT strftime('%s', datetime('now', 'utc'))
    ),
    (
        SELECT strftime('%s', datetime('now', 'utc'))
    )
FROM user;
DROP TABLE user;
ALTER TABLE up_user
    RENAME TO user;
PRAGMA foreign_keys = on;