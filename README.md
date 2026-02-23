<p align="center">
  <img src="/assets/images/logo.png" alt="App Logo" width="120" />
</p>

<h1 align="center">LearnCast</h1>

<p align="center">
  This is the LearnCast backend — a REST API server written in Rust. It handles authentication, content management, audio file delivery, and user progress tracking for the web admin panel and mobile clients (Android and iOS). 🦀
</p>

---

## 📋 Table of Contents

1. [Tech Stack at a Glance](#1-tech-stack-at-a-glance)
2. [Environment Configuration](#2-environment-configuration)
3. [Application Startup](#3-application-startup)
4. [Router Architecture](#4-router-architecture)
5. [Authentication & Middleware](#5-authentication--middleware)
6. [Module Structure](#6-module-structure)
7. [API Modules — Admin](#7-api-modules--admin)
8. [API Modules — User](#8-api-modules--user)
9. [File Service](#9-file-service)
10. [Database Layer](#10-database-layer)
11. [Error Handling](#11-error-handling)
12. [Localization](#12-localization)
13. [Utilities](#13-utilities)
14. [OpenAPI Documentation](#14-openapi-documentation)
15. [Docker & Deployment](#15-docker--deployment)
16. [Package Structure](#16-package-structure)

---

## 1. Tech Stack at a Glance

| Concern | Crate |
|---|---|
| HTTP framework | `axum` 0.8 + `tokio` 1.48 |
| Database | `sqlx` 0.8 (PostgreSQL, async, compile-time checked queries) |
| Cache / token store | `redis` 1.0 (async via `tokio-comp`) |
| Object storage | `aws-sdk-s3` 1.119 (Cloudflare R2 via custom endpoint) |
| Authentication | `jsonwebtoken` 10 (HS256) |
| Google Sign-In | `google-cloud-auth` 1.3 (ID token verification) |
| Telegram Sign-In | HMAC-SHA256 manual verification (`hmac` + `sha2` + `hex`) |
| Request validation | `validator` 0.20 (derive macros) |
| Serialization | `serde` + `serde_json` |
| Localization | `fluent-templates` 0.13 (Fluent FTL files) |
| OpenAPI docs | `utoipa` 5 + `utoipa-swagger-ui` 9 |
| Audio metadata | `ffmpeg-light` 0.2 (duration probe via presigned URL) |
| File type detection | `infer` 0.19 (magic bytes) |
| Cursor encoding | `postcard` 1 + `base64` 0.22 |
| UUID generation | `uuid` 1.19 (v4) |
| Error handling | `thiserror` 2 + `anyhow` 1 |
| HTTP tracing | `tower-http` 0.6 (`TraceLayer`, `CorsLayer`) |
| Timestamps | `time` 0.3 (RFC 3339 serde, `OffsetDateTime`) |

---

## 2. Environment Configuration

**File:** `src/config.rs`  
**Loading:** `LazyLock<AppConfig>` in `utils/mod.rs` — loaded once on first access, panics early if any variable is missing

All configuration is read from environment variables (`.env` file in debug builds via `dotenvy`):

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection string |
| `JWT_SECRET_ACCESS` | Secret key for signing access tokens (15 min TTL) |
| `JWT_SECRET_REFRESH` | Secret key for signing refresh tokens (7 day TTL) |
| `TELEGRAM_BOT_TOKEN` | Bot token used to verify Telegram login HMAC signatures |
| `R2_ENDPOINT_URL` | Cloudflare R2 S3-compatible endpoint |
| `R2_BUCKET_NAME` | R2 bucket name for audio and image files |
| `CLIENT_ORIGIN` | Allowed CORS origin (also used by the admin `origin_middleware`) |
| `BASE_PATH` | Base path prefix for the server (e.g. `/learncast`) |

---

## 3. Application Startup

**File:** `src/main.rs`

Startup sequence on `#[tokio::main]`:

1. Loads `.env` in debug builds via `dotenvy`
2. Initialises `tracing_subscriber` with `EnvFilter` set to `debug` for `tower_http`, `axum`, and `sqlx`
3. Creates a `PgPool` (max 10 connections) via `db::postgres::create_pool`
4. Runs embedded SQL migrations from `src/db/migrations/` via `sqlx::migrate!()`
5. Creates a Redis `Client` from `CONFIG.redis_url`
6. Loads AWS config from environment variables with `CONFIG.r2_endpoint_url` and region `"auto"` (Cloudflare R2 convention)
7. Creates `AppState { db, redis_client, s3_client }` and passes it to `build_app(state)`
8. Binds a `TcpListener` on `0.0.0.0:3000` and serves with `axum::serve`

---

## 4. Router Architecture

**File:** `src/app.rs`

`build_app(state: AppState) -> Router` assembles two independent sub-routers and merges them:

**Admin router** (`/v1/admin/*`) — protected by `origin_middleware` (checks `Origin` header matches `CLIENT_ORIGIN` in production):
- Auth, Author, Topic, Lesson routes
- SwaggerUI at `/admin/docs` with JSON spec at `/api-doc/admin/openapi.json`

**User router** (`/v1/user/*` and `/v1/file/*`) — has `cache_control_middleware` applied globally:
- Auth, Author, Topic, Lesson, Snip routes
- File routes (shared between admin/user, see [File Service](#9-file-service))
- SwaggerUI at `/user/docs` with JSON spec at `/api-doc/user/openapi.json`

Global layers applied on the root router:
- `TraceLayer::new_for_http()` — HTTP request/response tracing
- `CorsLayer` — allows `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS`; allows `Authorization`, `Content-Type`, `Accept` headers; allows credentials (`allow_credentials(true)`); restricts origin to `CLIENT_ORIGIN`

---

## 5. Authentication & Middleware

**File:** `src/middleware/auth.rs`

Four middleware functions, each implementing `axum::middleware::from_fn`:

| Middleware | Applied to | Token source | Role check |
|---|---|---|---|
| `user_auth_middleware` | All user routes | `Authorization: Bearer <token>` header | `claims.role == "user"` |
| `admin_auth_middleware` | All admin routes | `access_token` cookie | `claims.role == "admin"` |
| `common_auth_middleware` | `/v1/file/{path}` download | Bearer header **or** `access_token` cookie (fallback) | any valid role |
| `origin_middleware` | Admin sub-router | `Origin` header | must start with `CLIENT_ORIGIN` (skipped in debug builds) |

All middleware functions extract `AcceptLanguage` from the request, validate the JWT via `jwt::validate_access_token`, and insert `Claims` and `AcceptLanguage` into request extensions for downstream handlers.

**File:** `src/middleware/cache.rs`

`cache_control_middleware` — applied to the entire user router. Sets `Cache-Control: private, max-age=300` on all `GET` responses so client-side HTTP caches hold responses for 5 minutes.

### JWT (`src/utils/jwt.rs`)

- `generate(user_id, role)` — creates a matched pair: refresh token (7 days, signed with `JWT_SECRET_REFRESH`) and access token (15 min, signed with `JWT_SECRET_ACCESS`). Both use HS256
- `validate_access_token(token)` / `validate_refresh_token(token)` — decode and validate expiry
- `hash_token(token)` — SHA-256 hex digest of the raw token string, stored in `user_session.refresh_token_hash` so raw tokens never touch the database

### `AcceptLanguage` Extractor (`src/extractor/accept_language.rs`)

Implements `FromRequestParts`. Parses the first tag from the `Accept-Language` header, defaulting to `"en"`. Produces a `fluent_templates::LanguageIdentifier` used throughout error messages and service calls.

---

## 6. Module Structure

Source is organized into three module families under `src/module/`:

| Family | Prefix | Audience | Auth |
|---|---|---|---|
| `module::admin` | `/v1/admin` | Admin panel | Cookie-based, `role = "admin"` |
| `module::user` | `/v1/user` | Mobile clients | Bearer token, `role = "user"` |
| `module::common` | shared | Services and DTOs shared by both | depends on route |

Each resource (author, topic, lesson, snip) follows the same four-file pattern inside its module folder:

| File | Responsibility |
|---|---|
| `routes.rs` | Declares the `Router`, wires HTTP methods to handler functions, applies auth middleware layer |
| `controller.rs` | Handler functions — extracts state, validated inputs, calls services, returns `BaseResponse<T>` |
| `dto.rs` | Request and response structs with `serde`, `validator`, and `utoipa` derives |
| `mapper.rs` | Converts database entities to response DTOs |

The `module::common` family additionally has `service.rs` files that contain all shared business logic, called by both admin and user controllers.

---

## 7. API Modules — Admin

All admin routes are under `/v1/admin/` and require an `access_token` cookie with `role = "admin"`.

### Auth (`module/admin/auth/`)

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/admin/auth/signin` | Sign in with Telegram or Google data; sets `access_token` and `refresh_token` cookies |
| `POST` | `/v1/admin/auth/refresh-token` | Exchange refresh token for new token pair |
| `POST` | `/v1/admin/auth/logout` | Deletes the session row from `user_session` |
| `GET` | `/v1/admin/me` | Returns the authenticated admin user |

### Author (`module/admin/author/`)

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/admin/author` | Create author |
| `PUT` | `/v1/admin/author/{id}` | Update author |
| `GET` | `/v1/admin/author/{id}` | Get author by ID |
| `DELETE` | `/v1/admin/author/{id}` | Soft-delete (blocked if author has lessons) |
| `GET` | `/v1/admin/author` | Page authors (offset pagination: `page`, `limit`, `search`) |

### Topic (`module/admin/topic/`)

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/admin/topic` | Create topic under an author |
| `PUT` | `/v1/admin/topic/{id}` | Update topic |
| `GET` | `/v1/admin/topic/{id}` | Get topic by ID |
| `DELETE` | `/v1/admin/topic/{id}` | Soft-delete (blocked if topic has lessons) |
| `GET` | `/v1/admin/topic` | Page topics (offset pagination: `page`, `limit`, `author_id`, `search`) |

### Lesson (`module/admin/lesson/`)

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/admin/lesson` | Create lesson — probes audio duration and file size from R2 via `ffmpeg-light`, increments `author.lesson_count` and `topic.lesson_count` / `total_duration` in a transaction |
| `PUT` | `/v1/admin/lesson/{id}` | Update lesson — re-probes if `audio_path` changed |
| `GET` | `/v1/admin/lesson/{id}` | Get lesson by ID |
| `DELETE` | `/v1/admin/lesson/{id}` | Soft-delete — blocked if `listen_count >= 50` (`LessonDeleteTooManyListens`), decrements author/topic counters |
| `GET` | `/v1/admin/lesson` | Page lessons (offset pagination: `page`, `limit`, `author_id`, `topic_id`, `search`) |

---

## 8. API Modules — User

All user routes are under `/v1/user/` and require `Authorization: Bearer <access_token>` with `role = "user"`.

### Auth (`module/user/auth/`)

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/user/auth/signin` | Sign in with Telegram or Google. Upserts the user row, creates a `user_session`, returns `LoginResponse { user, credentials }` |
| `POST` | `/v1/user/auth/refresh-token` | Validates the refresh token, checks the session's `user_agent` matches, rotates both tokens |
| `POST` | `/v1/user/auth/logout` | Deletes all sessions for the user |

### Author (`module/user/author/`)

| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/user/author` | Cursor-paginated author list with optional `search` |
| `GET` | `/v1/user/author/deleted` | Authors soft-deleted since a given RFC 3339 timestamp (for client-side sync) |

### Topic (`module/user/topic/`)

| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/user/topic` | Cursor-paginated topic list with optional `author_id`, `search`, `sort`, `order` |
| `GET` | `/v1/user/topic/deleted` | Topics deleted since a timestamp |

### Lesson (`module/user/lesson/`)

| Method | Path | Description |
|---|---|---|
| `GET` | `/v1/user/lesson` | Cursor-paginated lesson list. Filters: `author_id`, `topic_id`, `search`, `status` (`not_started` / `in_progress` / `completed`), `favourite`, `sort`, `order`. Response includes `is_favourite` and `lesson_progress` per item |
| `GET` | `/v1/user/lesson/deleted` | Lessons deleted since a timestamp |
| `POST` | `/v1/user/lesson/{id}/listen` | Records a listen session by `session_id` (idempotent via `UNIQUE` constraint). Increments `lesson.listen_count` only on first insertion. Returns updated `listen_count` |
| `PATCH` | `/v1/user/lesson/{id}/progress` | Upserts a `lesson_progress` row (`started_at`, `last_position_ms`, `status`, `completed_at`). Also upserts `topic_progress.completed_lesson_count` if the lesson belongs to a topic |
| `POST` | `/v1/user/lesson/{id}/favourite` | Inserts a `favourite_lesson` row |
| `DELETE` | `/v1/user/lesson/{id}/favourite` | Deletes the `favourite_lesson` row |

### Snip (`module/user/snip/`)

| Method | Path | Description |
|---|---|---|
| `POST` | `/v1/user/lesson/{lesson_id}/snip` | Create a snip. Validates `end_ms - start_ms >= 10_000` (min 10 seconds). Identified by client-assigned `client_snip_id` (UUID). Returns the new snip with `user_snip_count` for the lesson |
| `PUT` | `/v1/user/lesson/snip/{client_snip_id}` | Update a snip. Returns `SnipNotOwnedUpdate` (403) if `user_id` doesn't match |
| `DELETE` | `/v1/user/lesson/snip/{client_snip_id}` | Soft-delete a snip. Returns `SnipNotOwnedDelete` (403) if not owner. Returns updated `user_snip_count` for the lesson |
| `GET` | `/v1/user/lesson/snip` | Cursor-paginated snip list for the authenticated user. Filters: `lesson_id`, `search`, `sort`, `order` |
| `GET` | `/v1/user/lesson/snip/deleted` | Snips deleted since a timestamp |
| `GET` | `/v1/user/lesson/{lesson_id}/snip/count` | Count of the authenticated user's snips for a specific lesson |

---

## 9. File Service

**Files:** `src/module/common/file/`

Routes are mounted at `/v1/file/` and serve three audiences:

| Route | Auth | Description |
|---|---|---|
| `GET /v1/file/image/{*file_path}` | Public (no auth) | Serves images directly from the local `uploads/image/` directory |
| `POST /v1/file/` | Admin cookie auth + `origin_middleware` | Multipart image upload (max 1 MB). Detected via `infer` magic bytes — audio files are rejected. Content-addressed storage: SHA-256 hash of bytes → filename (`uploads/image/<hash>.ext`) |
| `GET /v1/file/upload-url` | Admin cookie auth + `origin_middleware` | Generates a presigned R2 `PutObject` URL (1 minute TTL) for direct client-to-R2 audio uploads. Validates that the file is MP3 (MIME `audio/*` and `.mp3` extension) and under 100 MB. Returns `{ upload_url, file_key }` |
| `GET /v1/file/{*file_path}` | Any valid JWT (Bearer or cookie) | For `.mp3` files: generates a presigned R2 `GetObject` URL with TTL rounded up to the nearest 10 minutes of audio duration, returns HTTP 307 redirect with `Cache-Control: no-cache` headers. For other files: reads from `uploads/` directory and streams bytes with correct `Content-Type` |

**Audio duration presigning formula:** `((duration_in_minutes / 10) + 1) * 10` minutes. This ensures the presigned URL remains valid for the full duration of playback even if the user starts at the very beginning.

---

## 10. Database Layer

**File:** `src/db/`

PostgreSQL via `sqlx` with async queries and `PgPool` (max 10 connections). All queries use the `sqlx::query!` / `sqlx::query_as!` macros for compile-time SQL verification.

### Schema (`src/db/migrations/001_initial.sql`)

| Table | Key columns |
|---|---|
| `users` | `id`, `first_name`, `last_name`, `avatar_path`, `email`, `telegram_id`, `telegram_username`, `google_id`, `password_hash`, `is_admin`, soft-delete via `deleted_at` |
| `user_session` | `id`, `user_id`, `refresh_token_hash` (SHA-256 of raw token), `user_agent` |
| `author` | `id`, `name`, `avatar_path`, `lesson_count`; unique active index on `name` |
| `topic` | `id`, `author_id`, `title`, `description`, `cover_image_path`, `lesson_count`, `total_duration`, `snip_count`; unique active index on `(title, author_id)` |
| `lesson` | `id`, `author_id`, `topic_id`, `title`, `description`, `cover_image_path`, `audio_path`, `duration` (ms), `file_size` (bytes), `listen_count`, `snip_count`; unique active index on `(topic_id, title)` |
| `lesson_progress` | `user_id`, `lesson_id`, `started_at`, `last_position_ms`, `status` (enum: `not_started` / `in_progress` / `completed`), `completed_at`; unique on `(user_id, lesson_id)` |
| `topic_progress` | `user_id`, `topic_id`, `author_id`, `completed_lesson_count`; unique on `(user_id, topic_id)` |
| `favourite_lesson` | `user_id`, `lesson_id`; unique on `(user_id, lesson_id)` |
| `listen_session` | `session_id`, `user_id`, `lesson_id`; unique on `session_id` (deduplication guard) |
| `snip` | `id`, `client_snip_id` (unique UUID from client), `author_id`, `topic_id`, `lesson_id`, `user_id`, `start_ms`, `end_ms`, `note_text`, soft-delete via `deleted_at` |

All mutable tables have a `set_updated_at()` trigger that automatically updates `updated_at` on every `UPDATE`.

### Repositories (`src/db/<entity>/repo.rs`)

Each entity has a repository file with typed async functions. All write operations that touch multiple tables use `sqlx` transactions (`db.begin()` / `tx.commit()`). Example cross-table operations:

- `lesson::repo::insert` — inserts a lesson, then `topic::repo::update_stats` (lesson count, total duration) and `author::repo::update_stats` (lesson count), all in one transaction
- `lesson::repo::update_progress` — upserts `lesson_progress`, then calls `topic::repo::update_progress` to recount completed lessons in the topic

### Paging Strategies

Two strategies are used depending on the audience:

- **Offset pagination** (`PagingResponse<T>`) — used by admin endpoints. Returns `{ items, total, has_next }`. `total` is a separate `COUNT(*)` query
- **Cursor pagination** (`CursorPagingResponse<T>`) — used by user endpoints. The cursor is a `postcard`-serialized struct (e.g. `LessonCursor { id, snip_count, created_at }`) encoded as URL-safe base64. The service fetches `limit + 1` rows; if the extra row exists, it's removed and the last item's fields are encoded as `next_cursor`

---

## 11. Error Handling

**File:** `src/error/`

`AppError` is the unified error type returned by all handlers. It implements `IntoResponse`, converting each variant to the appropriate HTTP status code and a JSON body:

```json
{ "code": 100001, "message": "Not found", "data": null, "time": "2026-01-01T00:00:00Z" }
```

| Variant | HTTP Status | Code |
|---|---|---|
| `NotFound` | 404 | 100001 |
| `BadRequest { message }` | 400 | 100002 |
| `Internal` | 500 | 100003 |
| `UnsupportedFileType` | 415 | 100004 |
| `FileTooLarge` | 413 | 100005 |
| `Auth(AuthError::Unauthorized)` | 401 | 101001 |
| `Auth(AuthError::InvalidCredentials)` | 401 | 101002 |
| `Author(AuthorError::AuthorHasLesson)` | 409 | 102001 |
| `Topic(TopicError::TopicHasLesson)` | 409 | 103001 |
| `Lesson(LessonError::LessonDeleteTooManyListens)` | 409 | 104001 |
| `Snip(SnipError::SnipNotOwnedUpdate)` | 403 | 105001 |
| `Snip(SnipError::SnipNotOwnedDelete)` | 403 | 105002 |

`From<sqlx::Error>` and `From<anyhow::Error>` are implemented — both convert to `AppError::Internal`. The `anyhow` conversion walks the error chain looking for a downcastable `AppError` first.

Error messages are always resolved through the localization system using the `Accept-Language` from the request, so clients receive errors in their preferred language.

---

## 12. Localization

**Files:** `src/locales/en/strings.ftl`, `src/locales/uz/strings.ftl`

Fluent FTL files define all user-facing error messages. Loaded at startup via `fluent_templates::static_loader!` macro into a `LazyLock<Loader>`:

```rust
pub fn t(lang_id: &LanguageIdentifier, key: &str) -> String {
    LOCALES.lookup(lang_id, key)
}
```

### Build-time Key Generation (`build.rs`)

`build.rs` parses `src/locales/en/strings.ftl` using `fluent-syntax` and auto-generates `src/string_keys.rs` at compile time. Each FTL message ID becomes a `pub const` in `mod strings`:

```
not-found       →  strings::NOT_FOUND       = "not_found"
invalid-credentials  →  strings::INVALID_CREDENTIALS = "invalid_credentials"
```

This means all string key usages are checked at compile time — a missing or renamed FTL key causes a compilation error. `build.rs` is re-run automatically whenever `strings.ftl` changes.

---

## 13. Utilities

**File:** `src/utils/`

### Validated Extractors (`utils/mod.rs → extractors`)

Three custom axum `FromRequest` / `FromRequestParts` implementations that parse, validate, and return a typed value or an `AppError::BadRequest`:

- `ValidatedJson<T>` — parses JSON body and runs `validator::Validate`. Extracts the `Accept-Language` header from the raw request for error localisation before the body is consumed
- `ValidatedQuery<T>` — parses query parameters via `axum::extract::Query`
- `ValidatedPath<T>` — parses path parameters via `axum::extract::Path`

### Cursor Codec (`utils/cursor.rs`)

- `encode<T: Serialize>(value: T) -> Option<String>` — serialises with `postcard`, encodes as URL-safe base64 (no padding)
- `decode<T: DeserializeOwned>(cursor: Option<String>) -> Option<T>` — decodes and deserialises; returns `None` on any error so a malformed cursor is silently treated as "start from beginning"

### Telegram Login Verification (`utils/telegram.rs`)

`verify_telegram_login(data: &str, bot_token: &str) -> Result<TelegramAuthData>`:

1. Base64-decodes the `data` string (URL-safe, no padding)
2. Parses JSON and checks `auth_date` is within the last 15 seconds
3. Sorts all non-`hash` fields alphabetically, joins as `key=value\n` data check string
4. Derives the secret key as `SHA-256(bot_token)`
5. Computes `HMAC-SHA256(secret_key, data_check_string)` and hex-encodes
6. Compares with the `hash` field in the payload — returns `anyhow::Error` on mismatch

---

## 14. OpenAPI Documentation

**File:** `src/api_docs.rs`

Two separate `OpenApi` structs are generated via `utoipa`'s derive macros:

| Struct | Swagger UI URL | JSON URL | Security scheme |
|---|---|---|---|
| `UserApiDoc` | `/user/docs` | `/api-doc/user/openapi.json` | `bearerAuth` (HTTP Bearer JWT) |
| `AdminApiDoc` | `/admin/docs` | `/api-doc/admin/openapi.json` | `cookieAuth` (Cookie: `access_token`) |

All controllers are annotated with `#[utoipa::path(...)]` for full schema generation. All DTOs derive `ToSchema` and all query param structs derive `IntoParams`.

---

## 15. Docker & Deployment

### Dockerfile

Multi-stage build:

**Stage 1 — builder (`rust:1.91.1-bookworm`):**
1. Creates a dummy `src/main.rs` and builds dependencies only first — this layer is cached as long as `Cargo.toml`/`Cargo.lock` don't change
2. Restores the real `src/main.rs`, copies `build.rs` and `src/`, builds the release binary
3. Strips the binary with `strip` to minimise image size

**Stage 2 — runtime (`debian:bookworm-slim`):**
- Installs `ffmpeg` and `ca-certificates` (FFmpeg is required by `ffmpeg-light` for audio duration probing)
- Copies only the stripped binary from the builder stage
- Runs `./learncast`

### `compose.yaml`

Three services for local development:

| Service | Image | Port | Notes |
|---|---|---|---|
| `api` | `anaserkinov/learncast-api:latest` | `3000:3000` | Reads `.env`, mounts `./uploads` volume |
| `db` | `postgres:15-bookworm` | — | `POSTGRES_DB=learncast` |
| `redis` | `redis:alpine` | — | `--appendonly yes` for persistence |

PostgreSQL data and Redis data are persisted in named volumes (`postgresql_data`, `redis_data`).

---

## 16. Package Structure

```
learncast/
├── build.rs                  # Generates src/string_keys.rs from locales/en/strings.ftl
├── Cargo.toml
├── Dockerfile
├── compose.yaml
└── src/
    ├── main.rs               # Entry point: DB pool, migrations, Redis, S3, server bind
    ├── app.rs                # build_app(): assembles admin/user routers, CORS, tracing
    ├── config.rs             # AppConfig struct, reads from env vars
    ├── state.rs              # AppState { db, redis_client, s3_client }
    ├── api_docs.rs           # UserApiDoc + AdminApiDoc utoipa OpenApi structs
    ├── string_keys.rs        # Auto-generated string key constants (do not edit)
    ├── db/
    │   ├── postgres.rs       # create_pool()
    │   ├── migrations/       # SQL migration files (001_initial.sql)
    │   ├── user/             # entity.rs, repo.rs
    │   ├── session/          # entity.rs, repo.rs
    │   ├── author/           # entity.rs, repo.rs
    │   ├── topic/            # entity.rs, repo.rs
    │   ├── lesson/           # entity.rs, repo.rs
    │   └── snip/             # entity.rs, repo.rs
    ├── error/
    │   ├── mod.rs            # AppError enum, IntoResponse, From<sqlx::Error>, From<anyhow::Error>
    │   ├── auth.rs           # AuthError
    │   ├── author.rs         # AuthorError
    │   ├── topic.rs          # TopicError
    │   ├── lesson.rs         # LessonError
    │   └── snip.rs           # SnipError
    ├── extractor/
    │   └── accept_language.rs  # AcceptLanguage extractor
    ├── middleware/
    │   ├── auth.rs           # user/admin/common auth + origin middleware
    │   └── cache.rs          # cache_control_middleware (private, max-age=300)
    ├── utils/
    │   ├── mod.rs            # CONFIG, LOCALES, t(), ValidatedJson/Query/Path extractors
    │   ├── jwt.rs            # generate(), validate_*_token(), hash_token()
    │   ├── cursor.rs         # encode() / decode() cursor codec
    │   └── telegram.rs       # verify_telegram_login()
    ├── locales/
    │   ├── en/strings.ftl    # English error messages
    │   └── uz/strings.ftl    # Uzbek error messages
    └── module/
        ├── common/
        │   ├── base.rs       # BaseResponse<T>, FileResponse, IdParam, DeletedParams, etc.
        │   ├── paging.rs     # PagingResponse<T>, CursorPagingResponse<T>, QueryOrder
        │   ├── enums.rs      # UserProgressStatus
        │   ├── auth/         # service.rs (signin_with_telegram/google, refresh_tokens, logout)
        │   ├── author/       # service.rs, dto.rs, mapper.rs
        │   ├── topic/        # service.rs, dto.rs, mapper.rs
        │   ├── lesson/       # service.rs, dto.rs, mapper.rs
        │   └── file/         # routes.rs, controller.rs (upload, upload_url, download_file)
        ├── admin/
        │   ├── auth/         # routes.rs, controller.rs
        │   ├── author/       # routes.rs, controller.rs, dto.rs, mapper.rs
        │   ├── topic/        # routes.rs, controller.rs, dto.rs, mapper.rs
        │   └── lesson/       # routes.rs, controller.rs, dto.rs, mapper.rs
        └── user/
            ├── auth/         # routes.rs, controller.rs
            ├── author/       # routes.rs, controller.rs, dto.rs, mapper.rs
            ├── topic/        # routes.rs, controller.rs, dto.rs, mapper.rs
            ├── lesson/       # routes.rs, controller.rs, dto.rs, mapper.rs
            └── snip/         # routes.rs, controller.rs, dto.rs, mapper.rs, service.rs
```