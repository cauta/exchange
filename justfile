export DATABASE_URL := "postgresql://postgres:password@localhost:5432/exchange"

default:
  just --list

backend:
  cd apps/backend && cargo run

frontend:
  just types
  bun run dev

bots:
  cd apps/bots && cargo run

compose:
  docker compose up --build

# ================================

db:
  #!/usr/bin/env bash
  set -euo pipefail

  # Check if PostgreSQL is running
  if docker ps --filter name=exchange-postgres --format '{{{{.Names}}}}' | grep -q exchange-postgres; then
    echo "✅ PostgreSQL already running"
  else
    echo "🚀 Starting PostgreSQL..."
    docker compose up -d postgres
  fi

  # Check if ClickHouse is running
  if docker ps --filter name=exchange-clickhouse --format '{{{{.Names}}}}' | grep -q exchange-clickhouse; then
    echo "✅ ClickHouse already running"
  else
    echo "🚀 Starting ClickHouse..."
    docker compose up -d clickhouse
  fi

  echo "⏳ Waiting for databases to be ready..."
  sleep 3

  echo "📊 Running migrations..."
  just db-migrate

  echo "🔧 Initializing exchange..."
  just db-init

  echo "✅ Database is ready!"

db-run:
  docker compose up -d postgres clickhouse

db-stop:
  @docker compose down
  @echo "✅ Databases stopped (data preserved)"

db-init:
  # inits exchange with tokens and markets from config.toml
  # this also automatically sets up database schemas
  cd apps/backend && cargo run --bin init_exchange

db-migrate:
  cd apps/backend/src/db/pg && cargo sqlx migrate run --database-url $DATABASE_URL
  clickhouse client --user default --password password --query "$(cat apps/backend/src/db/ch/schema.sql)"

db-reset:
  @echo "🔄 Resetting databases (this will destroy all data)..."
  @docker compose down -v
  @echo "🚀 Starting fresh databases..."
  @docker compose up -d postgres clickhouse
  @echo "⏳ Waiting for databases to be ready..."
  @sleep 3
  @echo "📊 Running migrations..."
  @just db-migrate
  @echo "🔧 Initializing exchange..."
  @just db-init
  @echo "✅ Database reset complete!"

db-prepare:
  cd apps/backend && cargo sqlx prepare --database-url $DATABASE_URL

# ================================

install:
  bun install
  cd packages/sdk-python && uv sync

build:
  bun run build:sdk
  cargo build --workspace

test:
  cargo test --workspace

bench:
  cd apps/backend && cargo bench
  open target/criterion/report/index.html

# ================================

types:
  cd apps/backend && cargo run --bin generate_openapi
  cd apps/backend && cargo run --bin generate_websocket_schema
  bun --filter @exchange/sdk generate
  just types-python
  just fmt

types-python:
  #!/usr/bin/env bash
  cd packages/sdk-python && \
  mkdir -p exchange_sdk/generated && \
  uv run --with datamodel-code-generator[http] datamodel-codegen \
    --input ../../packages/shared/websocket.json \
    --input-file-type jsonschema \
    --output exchange_sdk/generated/websocket.py \
    --output-model-type pydantic_v2.BaseModel \
    --use-union-operator \
    --field-constraints \
    --use-standard-collections \
    --target-python-version 3.10 \
    --disable-timestamp && \
  echo "# Generated WebSocket types" > exchange_sdk/generated/__init__.py && \
  echo "from .websocket import *" >> exchange_sdk/generated/__init__.py

fmt:
  bun run format
  cargo fmt --all

lint:
  bun run lint
  cargo clippy --workspace --all-targets

typecheck:
  bun run typecheck

clean:
  bun run clean
  cargo clean

ci:
  just install
  just types
  just fmt
  just lint
  just db-prepare

sort:
  cargo sort -g -w

