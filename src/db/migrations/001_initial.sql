CREATE TYPE user_progress_status AS ENUM (
    'not_started',
    'in_progress',
    'completed'
    );

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- user
CREATE TABLE users
(
    id                BIGSERIAL PRIMARY KEY,
    first_name        TEXT        NOT NULL,
    last_name         TEXT,
    avatar_path       TEXT,
    email             TEXT,
    telegram_username TEXT,
    telegram_id       BIGINT,
    google_id         TEXT,
    password_hash     TEXT,
    is_admin          BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at        TIMESTAMPTZ
);

CREATE TRIGGER trg_users_set_updated_at
    BEFORE UPDATE
    ON users
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- session
CREATE TABLE user_session
(
    id                 BIGSERIAL PRIMARY KEY,
    user_id            BIGINT      NOT NULL,
    refresh_token_hash TEXT        NOT NULL,
    user_agent         TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER trg_user_session_set_updated_at
    BEFORE UPDATE
    ON user_session
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- author
CREATE TABLE author
(
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT        NOT NULL,
    avatar_path  TEXT,
    lesson_count BIGINT      NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at   TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_author_unique_active
    ON author (name)
    WHERE deleted_at IS NULL;

CREATE TRIGGER trg_author_set_updated_at
    BEFORE UPDATE
    ON author
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- topic
CREATE TABLE topic
(
    id               BIGSERIAL PRIMARY KEY,
    title            TEXT        NOT NULL,
    description      TEXT,
    cover_image_path TEXT,
    lesson_count     BIGINT      NOT NULL DEFAULT 0,
    total_duration   BIGINT      NOT NULL DEFAULT 0,
    snip_count       BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_topic_unique_active
    ON topic (title)
    WHERE deleted_at IS NULL;

CREATE TRIGGER trg_topic_set_updated_at
    BEFORE UPDATE
    ON topic
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();


CREATE TABLE author_topic
(
    id             BIGSERIAL PRIMARY KEY,
    author_id      BIGINT      NOT NULL,
    topic_id       BIGINT      NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    lesson_count   BIGINT      NOT NULL DEFAULT 0,
    snip_count     BIGINT      NOT NULL DEFAULT 0,
    total_duration BIGINT      NOT NULL DEFAULT 0,
    UNIQUE (author_id, topic_id)
);

-- lesson
CREATE TABLE lesson
(
    id               BIGSERIAL PRIMARY KEY,
    author_id        BIGINT      NOT NULL,
    topic_id         BIGINT,
    title            TEXT        NOT NULL,
    description      TEXT,
    cover_image_path TEXT,
    audio_path       TEXT        NOT NULL,
    duration         BIGINT      NOT NULL,
    file_size        BIGINT      NOT NULL,
    listen_count     BIGINT      NOT NULL DEFAULT 0,
    snip_count       BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_lesson_unique_active
    ON lesson (topic_id, author_id, title) NULLS NOT DISTINCT
    WHERE deleted_at IS NULL;

CREATE TRIGGER trg_lesson_set_updated_at
    BEFORE UPDATE
    ON lesson
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

CREATE TABLE lesson_progress
(
    id               BIGSERIAL PRIMARY KEY,
    user_id          BIGINT               NOT NULL,
    author_id        BIGINT               NOT NULL,
    topic_id         BIGINT,
    lesson_id        BIGINT               NOT NULL,
    started_at       TIMESTAMPTZ          NOT NULL,
    last_position_ms BIGINT               NOT NULL DEFAULT 0,
    status           user_progress_status NOT NULL DEFAULT 'in_progress',
    completed_at     TIMESTAMPTZ,
    UNIQUE (user_id, lesson_id)
);

CREATE TABLE author_topic_progress
(
    id                     BIGSERIAL PRIMARY KEY,
    user_id                BIGINT NOT NULL,
    author_id              BIGINT NOT NULL,
    topic_id               BIGINT NOT NULL,
    completed_lesson_count BIGINT NOT NULL DEFAULT 0,
    UNIQUE (user_id, author_id, topic_id)
);

CREATE TABLE favourite_lesson
(
    id        BIGSERIAL PRIMARY KEY,
    user_id   BIGINT NOT NULL,
    lesson_id BIGINT NOT NULL,
    UNIQUE (user_id, lesson_id)
);

CREATE TABLE listen_session
(
    id         BIGSERIAL PRIMARY KEY,
    session_id TEXT        NOT NULL,
    user_id    BIGINT      NOT NULL,
    lesson_id  BIGINT      NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (session_id)
);

CREATE TABLE snip
(
    id             BIGSERIAL PRIMARY KEY,
    client_snip_id TEXT        NOT NULL,
    author_id      BIGINT      NOT NULL,
    topic_id       BIGINT,
    lesson_id      BIGINT      NOT NULL,
    user_id        BIGINT      NOT NULL,
    start_ms       BIGINT      NOT NULL,
    end_ms         BIGINT      NOT NULL,
    note_text      TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ,
    UNIQUE (client_snip_id)
);

CREATE TRIGGER trg_snip_set_updated_at
    BEFORE UPDATE
    ON snip
    FOR EACH ROW
EXECUTE FUNCTION set_updated_at();