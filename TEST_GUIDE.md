# Exchange API Testing Guide

Quick guide to run the exchange backend locally and test with curl commands.

## Prerequisites

- Docker (for PostgreSQL and ClickHouse)
- Rust and Cargo
- `just` command runner (or use the commands manually)

## 1. Start the Databases

```bash
# Start PostgreSQL and ClickHouse containers
just db-run

# Wait for containers to be healthy (about 10 seconds)
# Then setup the database schema
just db-setup
```

Or manually:

```bash
docker compose up -d postgres clickhouse
sleep 10

# Run migrations
cd apps/backend/src/db/pg
cargo sqlx migrate run --database-url "postgresql://postgres:password@localhost:5432/exchange"

# Setup ClickHouse schema
clickhouse client --user default --password password --query "$(cat ../../ch/schema.sql)"
```

## 2. Start the Backend Server

```bash
# From project root
just backend

# Or manually
cd apps/backend
cargo run --release
```

The server will start on `http://localhost:8888` (default port).

**View API Documentation:** Open http://localhost:8888/api/docs in your browser.

---

## 3. Test with curl Commands

### Step 1: Check Health

```bash
curl http://localhost:8888/api/health
```

Expected: `{"status":"ok"}`

---

### Step 2: Create Tokens (Admin API)

**Create BTC token:**

```bash
curl -X POST http://localhost:8888/api/admin \
  -H "Content-Type: application/json" \
  -d '{
    "type": "create_token",
    "ticker": "BTC",
    "decimals": 18,
    "name": "Bitcoin"
  }'
```

**Create USDC token:**

```bash
curl -X POST http://localhost:8888/api/admin \
  -H "Content-Type: application/json" \
  -d '{
    "type": "create_token",
    "ticker": "USDC",
    "decimals": 18,
    "name": "USD Coin"
  }'
```

---

### Step 3: Create a Market (Admin API)

**Create BTC/USDC market:**

```bash
curl -X POST http://localhost:8888/api/admin \
  -H "Content-Type: application/json" \
  -d '{
    "type": "create_market",
    "base_ticker": "BTC",
    "quote_ticker": "USDC",
    "tick_size": "1000",
    "lot_size": "1000000",
    "min_size": "1000000",
    "maker_fee_bps": 10,
    "taker_fee_bps": 20
  }'
```

**Save the market_id from the response!** You'll need it for trading.

---

### Step 4: Fund User Accounts (Admin Faucet)

**Give Alice 10 BTC:**

```bash
curl -X POST http://localhost:8888/api/admin \
  -H "Content-Type: application/json" \
  -d '{
    "type": "faucet",
    "user_address": "alice",
    "token_ticker": "BTC",
    "amount": "10000000",
    "signature": "admin"
  }'
```

**Give Bob 100,000 USDC:**

```bash
curl -X POST http://localhost:8888/api/admin \
  -H "Content-Type: application/json" \
  -d '{
    "type": "faucet",
    "user_address": "bob",
    "token_ticker": "USDC",
    "amount": "100000000000",
    "signature": "admin"
  }'
```

---

### Step 5: Query Information

**Get all tokens:**

```bash
curl -X POST http://localhost:8888/api/info \
  -H "Content-Type: application/json" \
  -d '{"type": "all_tokens"}'
```

**Get all markets:**

```bash
curl -X POST http://localhost:8888/api/info \
  -H "Content-Type: application/json" \
  -d '{"type": "all_markets"}'
```

**Get specific token:**

```bash
curl -X POST http://localhost:8888/api/info \
  -H "Content-Type: application/json" \
  -d '{
    "type": "token_details",
    "ticker": "BTC"
  }'
```

**Get specific market (replace with your market_id):**

```bash
curl -X POST http://localhost:8888/api/info \
  -H "Content-Type: application/json" \
  -d '{
    "type": "market_details",
    "market_id": "YOUR_MARKET_ID_HERE"
  }'
```

---

### Step 6: Check User Balances

**Alice's balances:**

```bash
curl -X POST http://localhost:8888/api/user \
  -H "Content-Type: application/json" \
  -d '{
    "type": "balances",
    "user_address": "alice"
  }'
```

**Bob's balances:**

```bash
curl -X POST http://localhost:8888/api/user \
  -H "Content-Type: application/json" \
  -d '{
    "type": "balances",
    "user_address": "bob"
  }'
```

---

### Step 7: Place Orders

**Alice sells 1 BTC at $50,000:**

```bash
curl -X POST http://localhost:8888/api/trade \
  -H "Content-Type: application/json" \
  -d '{
    "type": "place_order",
    "user_address": "alice",
    "market_id": "YOUR_MARKET_ID_HERE",
    "side": "Sell",
    "order_type": "Limit",
    "price": "50000000000",
    "size": "1000000",
    "signature": "test_signature"
  }'
```

The response will include the order details. **Save the order_id** if you want to cancel it later.

**Bob buys 1 BTC at $50,000 (this will match!):**

```bash
curl -X POST http://localhost:8888/api/trade \
  -H "Content-Type: application/json" \
  -d '{
    "type": "place_order",
    "user_address": "bob",
    "market_id": "YOUR_MARKET_ID_HERE",
    "side": "Buy",
    "order_type": "Limit",
    "price": "50000000000",
    "size": "1000000",
    "signature": "test_signature"
  }'
```

When prices match, the order will execute immediately and you'll see the trades in the response!

---

### Step 8: View Orders and Trades

**Get Alice's orders:**

```bash
curl -X POST http://localhost:8888/api/user \
  -H "Content-Type: application/json" \
  -d '{
    "type": "orders",
    "user_address": "alice",
    "market_id": "YOUR_MARKET_ID_HERE"
  }'
```

**Get Alice's trade history:**

```bash
curl -X POST http://localhost:8888/api/user \
  -H "Content-Type: application/json" \
  -d '{
    "type": "trades",
    "user_address": "alice",
    "market_id": "YOUR_MARKET_ID_HERE"
  }'
```

---

### Step 9: Cancel an Order

**Cancel an order:**

```bash
curl -X POST http://localhost:8888/api/trade \
  -H "Content-Type: application/json" \
  -d '{
    "type": "cancel_order",
    "user_address": "alice",
    "order_id": "YOUR_ORDER_ID_HERE",
    "signature": "test_signature"
  }'
```

---

## Complete Trading Example

Here's a complete script to set up and execute a trade:

```bash
#!/bin/bash

BASE_URL="http://localhost:8888"

# 1. Create tokens
echo "Creating tokens..."
curl -s -X POST $BASE_URL/api/admin \
  -H "Content-Type: application/json" \
  -d '{"type":"create_token","ticker":"BTC","decimals":18,"name":"Bitcoin"}'

curl -s -X POST $BASE_URL/api/admin \
  -H "Content-Type: application/json" \
  -d '{"type":"create_token","ticker":"USDC","decimals":18,"name":"USD Coin"}'

# 2. Create market
echo -e "\n\nCreating market..."
MARKET_RESPONSE=$(curl -s -X POST $BASE_URL/api/admin \
  -H "Content-Type: application/json" \
  -d '{"type":"create_market","base_ticker":"BTC","quote_ticker":"USDC","tick_size":1000,"lot_size":1000000,"min_size":1000000,"maker_fee_bps":10,"taker_fee_bps":20}')

MARKET_ID=$(echo $MARKET_RESPONSE | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
echo "Market ID: $MARKET_ID"

# 3. Fund users
echo -e "\n\nFunding users..."
curl -s -X POST $BASE_URL/api/admin \
  -H "Content-Type: application/json" \
  -d '{"type":"faucet","user_address":"alice","token_ticker":"BTC","amount":"10000000","signature":"admin"}'

curl -s -X POST $BASE_URL/api/admin \
  -H "Content-Type: application/json" \
  -d '{"type":"faucet","user_address":"bob","token_ticker":"USDC","amount":"100000000000","signature":"admin"}'

# 4. Check balances
echo -e "\n\nAlice's balance:"
curl -s -X POST $BASE_URL/api/user \
  -H "Content-Type: application/json" \
  -d '{"type":"balances","user_address":"alice"}' | jq

echo -e "\n\nBob's balance:"
curl -s -X POST $BASE_URL/api/user \
  -H "Content-Type: application/json" \
  -d '{"type":"balances","user_address":"bob"}' | jq

# 5. Place orders
echo -e "\n\nAlice places sell order..."
curl -s -X POST $BASE_URL/api/trade \
  -H "Content-Type: application/json" \
  -d "{\"type\":\"place_order\",\"user_address\":\"alice\",\"market_id\":\"$MARKET_ID\",\"side\":\"Sell\",\"order_type\":\"Limit\",\"price\":\"50000000000\",\"size\":\"1000000\",\"signature\":\"test_sig\"}" | jq

echo -e "\n\nBob places buy order (will match!)..."
curl -s -X POST $BASE_URL/api/trade \
  -H "Content-Type: application/json" \
  -d "{\"type\":\"place_order\",\"user_address\":\"bob\",\"market_id\":\"$MARKET_ID\",\"side\":\"Buy\",\"order_type\":\"Limit\",\"price\":\"50000000000\",\"size\":\"1000000\",\"signature\":\"test_sig\"}" | jq

# 6. Check final balances
echo -e "\n\nAlice's final balance:"
curl -s -X POST $BASE_URL/api/user \
  -H "Content-Type: application/json" \
  -d '{"type":"balances","user_address":"alice"}' | jq

echo -e "\n\nBob's final balance:"
curl -s -X POST $BASE_URL/api/user \
  -H "Content-Type: application/json" \
  -d '{"type":"balances","user_address":"bob"}' | jq
```

Save this as `test_exchange.sh`, make it executable with `chmod +x test_exchange.sh`, and run it!

---

## Understanding the Numbers

The exchange uses fixed-point arithmetic with 18 decimals for precision:

- **BTC Amount**: `1000000` = 0.001 BTC = 1 milli-BTC
- **USDC Amount**: `50000000000` = 50,000 USDC
- **Price**: `50000000000` = $50,000 per BTC

To convert:

- Divide by `1_000_000_000_000_000_000` (1e18) to get decimal values
- Or use smaller test values like shown above

---

## WebSocket Testing

You can also connect to the WebSocket for real-time updates:

```bash
# Using websocat (install with: brew install websocat)
websocat ws://localhost:8888/ws
```

Then send:

```json
{ "type": "subscribe", "channel": "Trades", "market_id": "YOUR_MARKET_ID" }
```

---

## Cleanup

```bash
# Reset the database
just db-reset

# Or stop everything
docker compose down
docker compose down -v  # Include -v to remove volumes
```

---

## Troubleshooting

**Database connection errors?**

- Make sure PostgreSQL and ClickHouse containers are running: `docker ps`
- Check logs: `docker logs exchange-postgres` or `docker logs exchange-clickhouse`

**Port already in use?**

- Change the PORT environment variable or kill the process using port 8888

**Migration errors?**

- Try: `just db-reset` to reset and recreate the database

**Need to see backend logs?**

- Set `RUST_LOG=debug` before running: `RUST_LOG=debug just backend`
