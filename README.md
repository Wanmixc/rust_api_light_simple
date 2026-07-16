# rust_api_light_simple

Small Rust CRUD API for PostgreSQL, built with Axum + SQLx. JWT authentication (Argon2 hashed passwords).

## Endpoints

```text
# Public
GET    /health

# Auth
POST   /api/auth/register          { "username": "...", "password": "..." }
POST   /api/auth/login             { "username": "...", "password": "..." }

# Protected (Authorization: Bearer <token>)
GET    /api/items
POST   /api/items                  { "name": "...", "description": "..." }
GET    /api/items/{id}
PUT    /api/items/{id}             { "name": "...", "description": "..." }
DELETE /api/items/{id}
```

## Local Setup

Copy the example environment file and adjust the PostgreSQL values:

```bash
cp .env.example .env
```

Run the API:

```bash
nix develop -c cargo run
```

The server creates the required `items` and `users` tables on startup with an idempotent schema check.

## Environment

Use either `DATABASE_URL`:

```env
DATABASE_URL=postgres://user:password@host:5432/database?sslmode=require
```

Or split PostgreSQL values:

```env
DATABASE_HOST=localhost
DATABASE_USER=postgres
DATABASE_PASSWORD=postgres
DATABASE_NAME=rust_api_light_simple
DATABASE_SSL_MODE=disable
```

Runtime defaults:

```env
HOST=0.0.0.0
PORT=3010
MAX_DB_CONNECTIONS=2
JWT_SECRET=change-me-to-a-random-64-char-string
RUST_LOG=info
```

## Authentication Flow

1. **Register** a user:

```bash
curl -X POST http://localhost:3010/api/auth/register \
  -H 'content-type: application/json' \
  -d '{"username":"alice","password":"s3cret-p@ss"}'
```

Response (201):

```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "user": { "id": "uuid", "username": "alice" }
}
```

2. **Login** (get a new token):

```bash
curl -X POST http://localhost:3010/api/auth/login \
  -H 'content-type: application/json' \
  -d '{"username":"alice","password":"s3cret-p@ss"}'
```

3. **Use the token** for protected routes:

```bash
TOKEN="eyJhbGciOiJIUzI1NiJ9..."

# Create an item
curl -X POST http://localhost:3010/api/items \
  -H 'content-type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Notebook","description":"Daily notes"}'

# List items
curl http://localhost:3010/api/items \
  -H "Authorization: Bearer $TOKEN"

# Update an item
curl -X PUT http://localhost:3010/api/items/<id> \
  -H 'content-type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Updated","description":"Changed"}'

# Delete an item
curl -X DELETE http://localhost:3010/api/items/<id> \
  -H "Authorization: Bearer $TOKEN"
```

Health check (no auth required):

```bash
curl http://localhost:3010/health  # → "ok"
```

Tokens expire after **24 hours**. Passwords are hashed with **Argon2**.

## Deploy on Koyeb

This is a **Rust** app. Koyeb **buildpacks do not support Rust**, so deploy with the **Dockerfile** builder only.

### One-click (Dockerfile builder forced)

[![Deploy to Koyeb](https://www.koyeb.com/static/images/deploy/button.svg)](https://app.koyeb.com/deploy?name=rust-api-light-simple&type=git&builder=dockerfile&repository=Wanmixc/rust_api_light_simple&branch=main&ports=3010;http;/)

Direct link:

```text
https://app.koyeb.com/deploy?name=rust-api-light-simple&type=git&builder=dockerfile&repository=Wanmixc/rust_api_light_simple&branch=main&ports=3010;http;/
```

### Fix an existing service that keeps failing

Your current build log shows **Buildpack detection** (`heroku/*`, `No buildpack groups passed detection`). That means the service is still set to **Buildpack**, not **Dockerfile**.

1. Open the service on [Koyeb Control Panel](https://app.koyeb.com/).
2. Go to **Settings** → **Build and deployment** (or **Builder**).
3. Change **Builder** from **Buildpack** to **Dockerfile**.
4. Dockerfile path: `Dockerfile` (repo root).
5. Set **Port** to `3010` (or the same value as env `PORT`).
6. Health check path: `/health`.
7. Save and redeploy.

If there is no Builder toggle on the existing service, create a **new** service with the one-click link above (it forces `builder=dockerfile`).

### Environment variables

```env
DATABASE_URL=postgres://user:password@host:5432/database?sslmode=require
HOST=0.0.0.0
PORT=3010
JWT_SECRET=use-a-long-random-string-here
RUST_LOG=info
```

Or use split DB vars (`DATABASE_HOST`, `DATABASE_USER`, `DATABASE_PASSWORD`, `DATABASE_NAME`, `DATABASE_SSL_MODE`).

## Public Repo Notes

Do not commit `.env`. Commit `.env.example` instead and put real database credentials in the deployment provider's environment variable settings.