/// Raw HTTP test of admin endpoint
use exchange_test_utils::TestServer;

#[tokio::test]
async fn test_admin_raw_http() {
    let server = TestServer::start().await.expect("Failed to start server");

    // Create tokens first
    let client = reqwest::Client::new();

    let token_req = serde_json::json!({
        "type": "create_token",
        "ticker": "BTC",
        "decimals": 18,
        "name": "Bitcoin"
    });

    let resp = client
        .post(format!("{}/api/admin", server.base_url))
        .json(&token_req)
        .send()
        .await
        .expect("Failed to send request");

    println!("Token response status: {}", resp.status());
    println!("Token response: {}", resp.text().await.unwrap());

    // Create market
    let token_req2 = serde_json::json!({
        "type": "create_token",
        "ticker": "USDC",
        "decimals": 18,
        "name": "USD Coin"
    });

    client
        .post(format!("{}/api/admin", server.base_url))
        .json(&token_req2)
        .send()
        .await
        .expect("Failed to send request");

    let market_req = serde_json::json!({
        "type": "create_market",
        "base_ticker": "BTC",
        "quote_ticker": "USDC",
        "tick_size": 1000u128,
        "lot_size": 1000000u128,
        "min_size": 1000000u128,
        "maker_fee_bps": 10i32,
        "taker_fee_bps": 20i32,
    });

    let resp = client
        .post(format!("{}/api/admin", server.base_url))
        .json(&market_req)
        .send()
        .await
        .expect("Failed to send request");

    println!("Market response status: {}", resp.status());
    let body = resp.text().await.unwrap();
    println!("Market response body: {}", body);

    assert!(!body.is_empty(), "Response should not be empty");
}
