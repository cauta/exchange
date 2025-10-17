# Stage 1: Builder
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy package files
COPY package.json bun.lockb* ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Build
ENV NEXT_TELEMETRY_DISABLED=1
RUN bun run build

# Stage 2: Runtime
FROM oven/bun:1-slim

WORKDIR /app

# Copy built application from builder
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public

EXPOSE 3000

CMD ["bun", "server.js"]
