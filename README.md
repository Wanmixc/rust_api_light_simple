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

This project has no Node/Python/Go files, so **buildpacks will fail**. Deploy with the **Dockerfile** builder:

1. Create a new Koyeb service from this GitHub repo.
2. Set **Builder** to **Dockerfile** (not Buildpack).
3. Dockerfile path: `Dockerfile`.
4. Set the service HTTP port to match `PORT` (default in the image is `3010`, or use whatever `PORT` you set in env).
5. Add environment variables:

```env
DATABASE_URL=postgres://user:password@host:5432/database?sslmode=require
HOST=0.0.0.0
PORT=3010
RUST_LOG=info
```

Or use split DB vars (`DATABASE_HOST`, `DATABASE_USER`, `DATABASE_PASSWORD`, `DATABASE_NAME`, `DATABASE_SSL_MODE`).

Health check path: `/health`

## Public Repo Notes

Do not commit `.env`. Commit `.env.example` instead and put real database credentials in the deployment provider's environment variable settings.
