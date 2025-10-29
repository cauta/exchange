#!/bin/bash

echo "🚀 Setting up development environment..."

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
cd /workspace/apps/frontend && bun install

# Wait for databases to be ready
echo "⏳ Waiting for databases to be ready..."
until pg_isready -h postgres -p 5432 -U postgres; do
  echo "Waiting for PostgreSQL..."
  sleep 2
done

until clickhouse-client --host clickhouse --query "SELECT 1" > /dev/null 2>&1; do
  echo "Waiting for ClickHouse..."
  sleep 2
done

echo "✅ Databases are ready!"

# Run database migrations
echo "🗄️  Running database migrations..."
cd /workspace
just db-setup || echo "⚠️  Database setup failed. You may need to run 'just db-setup' manually."

# Build backend to check everything works
echo "🔨 Building backend..."
cd /workspace/apps/backend && cargo build || echo "⚠️  Backend build failed. You may need to fix compilation errors."

echo ""
echo "✅ Development environment is ready!"
echo ""
echo "📚 Available commands:"
echo "  just backend   - Run the backend server"
echo "  just frontend  - Run the frontend dev server"
echo "  just db-setup  - Set up databases"
echo "  just db-reset  - Reset databases"
echo "  just test      - Run tests"
echo "  just openapi   - Generate OpenAPI schema"
echo ""
