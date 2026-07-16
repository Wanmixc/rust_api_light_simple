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
PORT=3000
MAX_DB_CONNECTIONS=2
RUST_LOG=info
```

## Example Requests

```bash
curl http://localhost:3000/health
```

```bash
curl -X POST http://localhost:3000/api/items \
  -H 'content-type: application/json' \
  -d '{"name":"Notebook","description":"Daily notes"}'
```

```bash
curl http://localhost:3000/api/items
```

## Public Repo Notes

Do not commit `.env`. Commit `.env.example` instead and put real database credentials in the deployment provider's environment variable settings.
