# rust_api_light_simple

Small Rust CRUD API for PostgreSQL, built with Axum and SQLx.

## Endpoints

```text
GET    /health
GET    /api/items
POST   /api/items
GET    /api/items/{id}
PUT    /api/items/{id}
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

The server creates the required `items` table on startup with an idempotent schema check.

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
RUST_LOG=info
```

## Example Requests

```bash
curl http://localhost:3010/health
```

```bash
curl -X POST http://localhost:3010/api/items \
  -H 'content-type: application/json' \
  -d '{"name":"Notebook","description":"Daily notes"}'
```

```bash
curl http://localhost:3010/api/items
```

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
RUST_LOG=info
```

Or use split DB vars (`DATABASE_HOST`, `DATABASE_USER`, `DATABASE_PASSWORD`, `DATABASE_NAME`, `DATABASE_SSL_MODE`).

## Public Repo Notes

Do not commit `.env`. Commit `.env.example` instead and put real database credentials in the deployment provider's environment variable settings.
